#[tauri::command]
pub(crate) async fn fan_wallets(state: State<'_, AppState>) -> Result<WalletBatch, AppError> {
    let profile = fan_profile(&state).await?;
    let api = state.api.clone();
    let api_base_url = profile.api_base_url.clone();
    let requests = profile.wallets.iter().cloned().map(move |credential| {
        let api = api.clone();
        let api_base_url = api_base_url.clone();
        let expected_order_id = credential.order_id.clone();
        async move {
            let result = api
                .ticket_wallet(
                    &api_base_url,
                    &credential.order_id,
                    &credential.checkout_token,
                )
                .await
                .and_then(|value| {
                    if value.order.order_id.as_str() == credential.order_id.as_str() {
                        Ok(value)
                    } else {
                        Err(AppError::InvalidInput(
                            crate::i18n::tr("native_wrong_order_wallet").into(),
                        ))
                    }
                });
            (expected_order_id, result)
        }
    });
    let results = stream::iter(requests)
        .buffered(WALLET_FETCH_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;
    let request_count = results.len();
    let mut wallets = Vec::with_capacity(request_count);
    let mut wallet_tokens = Vec::with_capacity(request_count);
    let mut live_snapshots = Vec::with_capacity(request_count);
    let mut failed_orders = Vec::new();
    let mut first_error = None;
    for (order_id, result) in results {
        match result {
            Ok(wallet) => {
                let order_id = wallet.order.order_id.clone();
                let (wallet, tokens, qr_credentials) = prepare_wallet(wallet);
                live_snapshots.push((wallet.clone(), qr_credentials));
                wallets.push(wallet);
                wallet_tokens.push((order_id, tokens));
            }
            Err(error) => {
                failed_orders.push(order_id);
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
    }

    // Stronghold keeps the last public wallet snapshot plus only still-valid QR
    // credentials. Checkout secrets remain in their canonical credential list;
    // cached QR tokens stay encrypted/zeroized and never enter the WebView payload.
    let mut cached_count = 0usize;
    for order_id in &failed_orders {
        if let Some(mut cached) = profile
            .cached_wallets
            .iter()
            .find(|wallet| wallet.order.order_id.as_str() == order_id.as_str())
            .cloned()
        {
            cached.cached = true;
            for ticket in &mut cached.tickets {
                ticket.qr_available = profile.cached_wallet_qr.iter().any(|entry| {
                    entry.order_id.as_str() == order_id.as_str()
                        && entry.public_reference.as_str() == ticket.public_reference.as_str()
                        && wallet_qr_credential_valid(entry)
                });
            }
            wallets.push(cached);
            cached_count += 1;
        }
    }
    if wallets.is_empty()
        && let Some(error) = first_error
    {
        return Err(error);
    }

    let configured_orders = profile
        .wallets
        .iter()
        .map(|wallet| wallet.order_id.clone())
        .collect::<std::collections::HashSet<_>>();
    let mut cached_tokens = state.wallet_qr_tokens.write().await;
    cached_tokens.retain(|order_id, _| configured_orders.contains(order_id));
    cached_tokens.extend(wallet_tokens);
    drop(cached_tokens);

    if !live_snapshots.is_empty() {
        let _mutation = state.fan_mutation.lock().await;
        let latest = fan_profile(&state).await?;
        if latest.fan_session_token == profile.fan_session_token {
            let configured = latest
                .wallets
                .iter()
                .map(|wallet| wallet.order_id.clone())
                .collect::<std::collections::HashSet<_>>();
            let mut updated = latest.as_ref().clone();
            updated
                .cached_wallets
                .retain(|wallet| configured.contains(&wallet.order.order_id));
            for (mut snapshot, qr_credentials) in live_snapshots {
                snapshot.cached = false;
                let order_id = snapshot.order.order_id.clone();
                updated
                    .cached_wallets
                    .retain(|wallet| wallet.order.order_id.as_str() != order_id.as_str());
                updated.cached_wallets.push(snapshot);
                updated
                    .cached_wallet_qr
                    .retain(|entry| entry.order_id.as_str() != order_id.as_str());
                updated.cached_wallet_qr.extend(qr_credentials);
            }
            if updated.cached_wallets.len() > MAX_WALLETS {
                updated.cached_wallets.truncate(MAX_WALLETS);
            }
            updated.cached_wallet_qr.retain(wallet_qr_credential_valid);
            if updated.cached_wallet_qr.len() > MAX_WALLETS.saturating_mul(8) {
                updated
                    .cached_wallet_qr
                    .truncate(MAX_WALLETS.saturating_mul(8));
            }
            persist_fan(&state, &updated).await?;
            *state.fan_session.write().await = Some(Arc::new(updated));
        }
    }

    Ok(WalletBatch {
        failed_count: failed_orders.len(),
        cached_count,
        wallets,
    })
}

#[tauri::command]
pub(crate) async fn render_wallet_qr(
    state: State<'_, AppState>,
    order_id: String,
    public_reference: String,
) -> Result<String, AppError> {
    let order_id = uuid::Uuid::parse_str(order_id.trim())
        .map(|value| value.to_string())
        .map_err(|_| AppError::InvalidInput(crate::i18n::tr("native_invalid_order_id").into()))?;
    let public_reference = public_reference.trim();
    if public_reference.is_empty() || public_reference.len() > 200 {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_ticket_reference_invalid").into(),
        ));
    }
    let live_token = state
        .wallet_qr_tokens
        .read()
        .await
        .get(&order_id)
        .and_then(|tickets| tickets.get(public_reference))
        .cloned();
    let token = match live_token {
        Some(token) => token,
        None => {
            let profile = fan_profile(&state).await?;
            profile
                .cached_wallet_qr
                .iter()
                .find(|entry| {
                    entry.order_id.as_str() == order_id.as_str()
                        && entry.public_reference.as_str() == public_reference
                        && wallet_qr_credential_valid(entry)
                })
                .map(|entry| Zeroizing::new(entry.token.clone()))
                .ok_or(AppError::NotFound)?
        }
    };
    run_blocking(move || render_qr(token.as_str())).await
}

fn prepare_wallet(
    wallet: TicketWalletApi,
) -> (
    TicketWallet,
    HashMap<String, Zeroizing<String>>,
    Vec<WalletQrCredential>,
) {
    let order_id = wallet.order.order_id.clone();
    let mut tokens = HashMap::with_capacity(wallet.tickets.len());
    let mut cached_qr = Vec::with_capacity(wallet.tickets.len());
    let tickets = wallet
        .tickets
        .into_iter()
        .map(|ticket| {
            let qr_available = match ticket.qr_token {
                Some(token) => {
                    let credential = WalletQrCredential {
                        order_id: order_id.clone(),
                        public_reference: ticket.public_reference.clone(),
                        token,
                        expires_at: ticket.qr_expires_at.clone(),
                    };
                    if wallet_qr_credential_valid(&credential) {
                        tokens.insert(
                            ticket.public_reference.clone(),
                            Zeroizing::new(credential.token.clone()),
                        );
                        cached_qr.push(credential);
                        true
                    } else {
                        false
                    }
                }
                None => false,
            };
            WalletTicket {
                ticket_type_name: ticket.ticket_type_name,
                public_reference: ticket.public_reference,
                holder_name: ticket.holder_name,
                holder_email_masked: ticket.holder_email_masked,
                status: ticket.status,
                redeemed_at: ticket.redeemed_at,
                qr_available,
                qr_expires_at: ticket.qr_expires_at,
            }
        })
        .collect();
    cached_qr.retain(wallet_qr_credential_valid);
    (
        TicketWallet {
            order: wallet.order,
            tickets,
            cached: false,
        },
        tokens,
        cached_qr,
    )
}

fn wallet_qr_credential_valid(value: &WalletQrCredential) -> bool {
    use time::format_description::well_known::Rfc3339;
    if value.token.is_empty() || value.token.len() > MAX_SECRET_BYTES {
        return false;
    }
    time::OffsetDateTime::parse(&value.expires_at, &Rfc3339)
        .is_ok_and(|expires_at| expires_at > time::OffsetDateTime::now_utc())
}

#[tauri::command]
pub(crate) async fn fan_request_delivery(
    state: State<'_, AppState>,
    order_id: String,
) -> Result<serde_json::Value, AppError> {
    let order_id = uuid::Uuid::parse_str(order_id.trim())
        .map(|value| value.to_string())
        .map_err(|_| AppError::InvalidInput(crate::i18n::tr("native_invalid_order_id").into()))?;
    let profile = fan_profile(&state).await?;
    let wallet = profile
        .wallets
        .iter()
        .find(|wallet| wallet.order_id.as_str() == order_id.as_str())
        .ok_or_else(|| {
            AppError::InvalidInput(crate::i18n::tr("native_ticket_not_on_device").into())
        })?;
    state
        .api
        .request_ticket_delivery(
            &profile.api_base_url,
            &wallet.order_id,
            &wallet.checkout_token,
        )
        .await
}

fn attach_single_qr(value: &mut serde_json::Value) -> Result<(), AppError> {
    let token = value
        .get("token")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| AppError::InvalidInput(crate::i18n::tr("native_qr_token_missing").into()))?;
    let svg = render_qr(token)?;
    value["qr_svg"] = serde_json::Value::String(svg);
    Ok(())
}

fn render_qr(token: &str) -> Result<String, AppError> {
    if token.is_empty() || token.len() > MAX_SECRET_BYTES {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_qr_token_invalid").into(),
        ));
    }
    let code = QrCode::new(token.as_bytes()).map_err(|_| {
        AppError::InvalidInput(crate::i18n::tr("native_qr_generation_failed").into())
    })?;
    let rendered = code
        .render::<svg::Color>()
        .min_dimensions(320, 320)
        .dark_color(svg::Color("#080808"))
        .light_color(svg::Color("#ffffff"))
        .build();

    // qrcode's SVG renderer prepends an XML declaration. The webview contract
    // expects a standalone <svg> fragment suitable for direct DOM insertion.
    let start = rendered.find("<svg").ok_or_else(|| {
        AppError::InvalidInput(crate::i18n::tr("native_qr_generation_failed").into())
    })?;
    let svg = rendered[start..].trim();
    if !svg.ends_with("</svg>") {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_qr_generation_failed").into(),
        ));
    }
    Ok(svg.to_owned())
}

