const STAGE_PACK_PREVIEW_URL: &str = "bundle-stage-pack.webp";

#[component]
fn FanMerch(
    merch: RwSignal<Option<MerchCatalog>>,
    bundles: RwSignal<Option<FanMerchBundleCatalog>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <section class="screen fan-screen merch-screen">
            <header class="screen-title">
                <p class="eyebrow">{tr("virya_store")}</p>
                <h2>{tr("merch")}</h2>
                <p>{tr("products_and_bundles_use_the_same_inventory")}</p>
            </header>
            <Show when=move || !loading.get().merch fallback=move || view! { <Skeleton rows=4 /> }>
                {move || merch.get().map(|catalog| {
                    let products = catalog.products.into_iter()
                        .filter(|product| product.active && product.public)
                        .collect::<Vec<_>>();
                    if products.is_empty() {
                        view! {
                            <div class="empty-state">
                                <strong>{tr("store_is_temporarily_unavailable")}</strong>
                                <p>{tr("rest_of_signal_is_working_normally_try")}</p>
                                <button class="ghost" on:click=move |_| {
                                    refresh_fan_merch(merch, loading, error);
                                    refresh_fan_merch_bundles(bundles);
                                }>{tr("refresh_merch")}</button>
                            </div>
                        }.into_any()
                    } else {
                        let bundle_catalog = bundles.get();
                        view! {
                            <div class="fan-merch-list">
                                <div class="merch-grid-action">
                                    <ExternalLink url="https://virya.music/pl/merch/?source=signal-app".to_owned() label=tr("open_full_store") error=error />
                                </div>
                                <div class="merch-grid-heading">
                                    <div><p class="eyebrow">{tr("bundles")}</p><h3>{tr("bundles_from_the_online_store")}</h3></div>
                                    <span>{tr("up_to_30")}</span>
                                </div>
                                {bundle_catalog.map(|catalog| {
                                    if catalog.bundles.is_empty() {
                                        view! {
                                            <div class="merch-grid-message">
                                                <p>{tr("bundles_are_currently_unavailable_in_live_inventory")}</p>
                                                <ExternalLink url="https://virya.music/pl/merch/?source=signal-app&product=bundle-stage-pack".to_owned() label=tr("view_bundles") error=error />
                                            </div>
                                        }.into_any()
                                    } else {
                                        catalog.bundles.into_iter().map(|bundle| {
                                            let availability_label = match bundle.availability.as_str() {
                                                "low_stock" => {tr("low_stock")},
                                                "available" => {tr("available_status")},
                                                _ => {tr("out_of_stock")},
                                            };
                                            let available = bundle.available;
                                            let product_url = bundle.product_url.clone();
                                            let image_url = if bundle.slug == "bundle-stage-pack" {
                                                Some(STAGE_PACK_PREVIEW_URL.to_owned())
                                            } else {
                                                bundle.image_url
                                            };
                                            let bundle_name = bundle.name;
                                            let image_alt = i18n::format(
                                                "value_zestaw_merchu_virya",
                                                std::slice::from_ref(&bundle_name),
                                            );
                                            let original_price = (bundle.original_price_gross_minor > bundle.price_gross_minor)
                                                .then(|| money(bundle.original_price_gross_minor, &bundle.currency));
                                            let includes = bundle.includes;
                                            let includes_view = (!includes.is_empty()).then(|| view! {
                                                <ul class="fan-merch-includes">
                                                    {includes.into_iter().map(|item| view! { <li>{item}</li> }).collect_view()}
                                                </ul>
                                            });
                                            let variants = bundle.variants;
                                            let variants_view = (!variants.is_empty()).then(|| view! {
                                                <div class="fan-merch-variants">
                                                    {variants.into_iter().map(|variant| view! {
                                                        <span class:available=variant.available>{variant.label}</span>
                                                    }).collect_view()}
                                                </div>
                                            });
                                            view! {
                                                <article class="fan-merch-card fan-merch-bundle">
                                                    <div class="bundle-badge">"BUNDLE"</div>
                                                    {image_url.map(|url| view! {
                                                        <img src=url alt=image_alt width="720" height="720" loading="lazy" decoding="async" referrerpolicy="no-referrer" />
                                                    })}
                                                    <div class="fan-merch-body">
                                                        <div class="fan-merch-heading">
                                                            <div>
                                                                <h3>{bundle_name}</h3>
                                                                <div class="fan-merch-price">
                                                                    <strong>{money(bundle.price_gross_minor, &bundle.currency)}</strong>
                                                                    {original_price.map(|price| view! { <del>{price}</del> })}
                                                                </div>
                                                            </div>
                                                            <span class:available=available>{availability_label}</span>
                                                        </div>
                                                        {bundle.description.map(|description| view! { <p>{description}</p> })}
                                                        {includes_view}
                                                        {variants_view}
                                                        <Show when=move || available fallback=move || view! {
                                                            <button class="ghost" on:click=move |_| refresh_fan_merch_bundles(bundles)>{tr("check_again")}</button>
                                                        }>
                                                            <ExternalLink url=product_url.clone() label=tr("buy_in_store") error=error />
                                                        </Show>
                                                    </div>
                                                </article>
                                            }
                                        }).collect_view().into_any()
                                    }
                                }).value_or_else(|| view! {
                                    <div class="merch-grid-message">
                                        <p>{tr("bundles_load_independently_from_products")}</p>
                                        <ExternalLink url="https://virya.music/pl/merch/?source=signal-app&product=bundle-stage-pack".to_owned() label=tr("view_bundles") error=error />
                                    </div>
                                }.into_any())}
                                <div class="merch-grid-heading merch-products-heading">
                                    <div><p class="eyebrow">{tr("individual_products")}</p><h3>{tr("choose_your_merch")}</h3></div>
                                </div>
                                {products.into_iter().map(|product| {
                                    let available_variants = product.variants.iter()
                                        .filter(|variant| variant.active && variant.available)
                                        .cloned()
                                        .collect::<Vec<_>>();
                                    let has_stock = !available_variants.is_empty();
                                    let preorder = available_variants.iter()
                                        .any(|variant| variant.availability == "preorder");
                                    let low_stock = available_variants.iter()
                                        .any(|variant| variant.availability == "low_stock");
                                    let availability_label = if preorder {
                                        tr("pre_order")
                                    } else if low_stock {
                                        tr("low_stock")
                                    } else if has_stock {
                                        tr("available_status")
                                    } else {
                                        tr("out_of_stock")
                                    };
                                    let shop_url = format!(
                                        "https://virya.music/pl/merch/?source=signal-app&product={}",
                                        product.slug,
                                    );
                                    let image_url = if product.slug == "echoes" {
                                        Some(STAGE_PACK_PREVIEW_URL.to_owned())
                                    } else {
                                        product.image_url
                                    };
                                    let product_name = product.name;
                                    let image_alt = i18n::format(
                                        "value_merch_virya",
                                        std::slice::from_ref(&product_name),
                                    );
                                    let variants = product.variants.into_iter()
                                        .filter(|variant| variant.active)
                                        .collect::<Vec<_>>();
                                    let variants_view = (!variants.is_empty()).then(|| view! {
                                        <div class="fan-merch-variants">
                                            {variants.into_iter().map(|variant| view! {
                                                <span class:available=variant.available>{variant.label}</span>
                                            }).collect_view()}
                                        </div>
                                    });
                                    view! {
                                        <article class="fan-merch-card">
                                            {image_url.map(|url| view! {
                                                <img src=url alt=image_alt width="720" height="720" loading="lazy" decoding="async" referrerpolicy="no-referrer" />
                                            })}
                                            <div class="fan-merch-body">
                                                <div class="fan-merch-heading">
                                                    <div><h3>{product_name}</h3><strong>{money(product.price_gross_minor, &product.currency)}</strong></div>
                                                    <span class:available=has_stock>{availability_label}</span>
                                                </div>
                                                {product.description.map(|description| view! { <p>{description}</p> })}
                                                {variants_view}
                                                <Show when=move || has_stock fallback=move || view! {
                                                    <button class="ghost" on:click=move |_| refresh_fan_merch(merch, loading, error)>{tr("check_again")}</button>
                                                }>
                                                    <ExternalLink url=shop_url.clone() label=tr("buy_in_store") error=error />
                                                </Show>
                                            </div>
                                        </article>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }).value_or_else(|| view! {
                    <div class="empty-state">
                        <strong>{tr("could_not_load_store_status")}</strong>
                        <p>{tr("shows_tickets_and_profile_remain_available")}</p>
                        <button class="ghost" on:click=move |_| {
                            refresh_fan_merch(merch, loading, error);
                            refresh_fan_merch_bundles(bundles);
                        }>{tr("try_again")}</button>
                    </div>
                }.into_any())}
            </Show>
        </section>
    }
}
