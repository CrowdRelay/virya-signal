pub(super) fn text(key: &'static str) -> &'static str {
    match key {
        "latarnik_zone" => "BEACON ZONE",
        "latarnik_short_pitch" => {
            "Media, radio, photographers and local partners: shows in your radius and press materials without digging through email."
        }
        "open_latarnik" => "OPEN BEACON",
        "back_signal" => "← SIGNAL",
        "are_you_on_the_staff" => "ARE YOU ON THE STAFF?",
        "enter_the_staff_password_used_in_the" => {
            "Enter the staff password used in the Virya panel."
        }
        "zone_prefix" => "The ",
        "team_zone_suffix" => "team zone.",
        "gate_sales_and_show_operations_access_is" => {
            "Gate, sales and show operations access is separate from the fan account."
        }
        "staff_verification" => "STAFF VERIFICATION",
        "virya_panel_password" => "Virya panel password",
        "use_the_same_password_as_in_qr" => {
            "Use the same password as in QR, Gate and Control Center. After verification, the app will show the local PIN or device pairing."
        }
        "staff_password" => "Staff password",
        "checking" => "CHECKING…",
        "open_staff_zone" => "OPEN STAFF ZONE",
        "password_is_verified_by_virya_music_and" => {
            "The password is verified by virya.music and is not stored in the app."
        }
        "failed_to_read_the_staff_vault" => "FAILED TO READ THE STAFF VAULT",
        "checking_the_secure_vault" => "CHECKING THE SECURE VAULT",
        "virya_staff" => "Virya staff",
        "pin_must_contain_at_least_4_characters" => "The PIN must contain at least 4 characters.",
        "enter_a_4_6_digit_pin_and" => "Enter a 4–6 digit PIN and scan or paste the pairing code.",
        "enter_a_4_6_digit_pin_and_2" => "Enter a 4–6 digit PIN and a valid device token.",
        "code_scanned" => "Code scanned",
        "enter_the_pin_below_and_tap_pair" => "Enter the PIN below and tap PAIR.",
        "connecting" => "CONNECTING…",
        "scan_qr_code" => "SCAN QR CODE",
        "code_shown_in_the_virya_panel" => "Code shown in the Virya panel",
        "or_label" => "OR",
        "pairing_code" => "Pairing code",
        "create_an_unlock_pin" => "Create an unlock PIN",
        "enter_4_6_digits_for_example_2580" => {
            "Enter 4–6 digits, for example 2580. This PIN unlocks Virya Signal only — it is not the QR code or your phone PIN."
        }
        "pin_example" => "e.g. 2580",
        "hide_manual_settings" => "HIDE MANUAL SETTINGS",
        "advanced_settings" => "ADVANCED SETTINGS",
        "device_person_name" => "Device / person name",
        "device_token" => "Device token",
        "save_manually" => "SAVE MANUALLY",
        "cancel" => "CANCEL",
        "save" => "SAVE",
        "app_unlock_pin" => "App unlock PIN",
        "enter_the_pin_created_when_this_device" => {
            "Enter the PIN created when this device was paired."
        }
        "your_pin" => "Your PIN",
        "unlock" => "UNLOCK",
        "open_menu" => "Open menu",
        "close_and_lock_panel" => "Close and lock panel",
        "discounts" => "Discounts",
        "qr_codes" => "QR codes",
        "settings" => "Settings",
        "home_tab" => "Home",
        "album_experience" => "Album experience",
        "synesthesia_five_album_draw" => {
            "Walk through all 11 rooms. One completion gives 1 entry in a separate five-album draw."
        }
        "enter_synesthesia" => "ENTER SYNESTHESIA",
        "synesthesia_best_time" => "Best time {}",
        "synesthesia_rank" => "rank #{}",
        "synesthesia_rooms_progress" => "{}/11 rooms",
        "synesthesia_rooms_done" => "{}/11 rooms · completed",
        "synesthesia_runs_count" => "{} attempts",
        "synesthesia_result_saved_in_signal" => "Synesthesia result saved in Signal.",
        "synesthesia_handoff_expired_retry" => {
            "The Synesthesia link expired. Return to the finale and choose Connect to Signal again."
        }
        "your_signal_now" => "YOUR SIGNAL NOW",
        "your_participation" => "YOUR TRACE",
        "participation_history_title" => "What you have been part of",
        "participation_history_hint" => "Participation history, not points or levels.",
        "synesthesia_journey" => "Synesthesia journey",
        "area_discoveries" => "AREA discoveries",
        "concert_orders" => "ticket orders",
        "concert_passes" => "admission passes",
        "signal_city_context" => "Nearest context: {}.",
        "signal_home_context" => "One place for shows, tickets, AREA and album progress.",
        "cached_data" => "CACHED DATA",
        "wallet_cached_offline" => "OFFLINE — ENCRYPTED WALLET COPY",
        "journey_completed" => "Journey completed",
        "continue_the_journey" => "Continue the journey",
        "start_the_journey" => "Start the journey",
        "completion_linked_to_signal" => "This completion is already linked to your Signal.",
        "completion_saved_link_it_to_signal" => {
            "Completion saved. Open Synesthesia to link it to your profile."
        }
        "rooms_completed_count" => "Consciousness rooms completed in this journey.",
        "open_synesthesia" => "OPEN SYNESTHESIA",
        "next_signal" => "NEXT SIGNAL",
        "show_details" => "DETAILS",
        "active_passes" => "active passes",
        "area_findings" => "AREA finds",
        "signal_home_unavailable" => "The Signal snapshot is temporarily unavailable",
        "signal_home_fallback_hint" => {
            "The other tabs still work independently. Try refreshing again shortly."
        }
        "signal_tab" => "Signal",
        "scan_tab" => "Scan",
        "tickets_tab" => "Tickets",
        "shows_tab" => "Shows",
        "store_tab" => "Store",
        "profile_tab" => "Profile",
        "area_game_tab" => "AREA game",
        "staff_zone" => "Staff zone",
        "close_and_lock_signal" => "Close and lock Signal",
        "shows_count_label" => "shows",
        "active_qr" => "active QR",
        "check_ins" => "check-ins",
        "no_upcoming_shows" => "No upcoming shows",
        "new_events_will_appear_here" => "New events will appear here.",
        "owner_only_view" => "Owner-only view",
        "consent_growth_and_city_statistics_are_available" => {
            "Consent, growth and city statistics are available to the owner only."
        }
        "data_is_aggregated_in_crowdrelay_and_contains" => {
            "Data is aggregated in CrowdRelay and contains no email addresses or fan identifiers."
        }
        "refreshing" => "REFRESHING…",
        "refresh" => "REFRESH",
        "no_signal_snapshot" => "No Signal snapshot",
        "refresh_the_data_if_the_backend_is" => {
            "Refresh the data. If the backend is still being deployed, the panel will show a safe error instead of an empty screen."
        }
        "partial_snapshot_unavailable_sources" => "Partial snapshot. Unavailable sources: {}.",
        "no_city_aggregate" => "No city aggregate",
        "signal_has_no_confirmed_city_data_yet" => {
            "Signal has no confirmed city data yet, or the source is temporarily unavailable."
        }
        "active" => "active",
        "marketing_consents" => "marketing consents",
        "new_30_days" => "new / 30 days",
        "confirmed_among_active_and_pending" => "confirmed among active and pending",
        "all" => "all",
        "pending" => "pending",
        "unsubscribed" => "unsubscribed",
        "muted" => "muted",
        "nearby_notifications" => "nearby notifications",
        "activity" => "Activity",
        "text_30_days_total" => "30 days / total",
        "new_7_days" => "new / 7 days",
        "referrals" => "referrals",
        "show_interests" => "show interests",
        "nearby_notifications_2" => "nearby notifications",
        "cities_awaiting_moderation" => "cities awaiting moderation",
        "strongest_cities" => "Strongest cities",
        "snapshot_generated_at_aggregated_data_only" => {
            "Snapshot: {generated_at}. Aggregated data only."
        }
        "select_a_show_first" => "Select a show first.",
        "select_a_show" => "Select a show.",
        "scan_status_redeemed" => "REDEEMED",
        "scan_status_already_redeemed" => "ALREADY REDEEMED",
        "scan_status_revoked" => "REVOKED — DO NOT ADMIT",
        "scan_status_expired" => "EXPIRED — DO NOT ADMIT",
        "scan_status_not_claimed" => "NOT CLAIMED — DO NOT ADMIT",
        "scan_status_offline_queued" => "QUEUED OFFLINE",
        "scan_status_offline_duplicate" => "ALREADY SCANNED (OFFLINE)",
        "snapshot_ready_durable_tickets" => "Snapshot ready: {} durable tickets.",
        "sync_saved_conflicts_still_pending" => "Sync: {} saved, {} conflicts, {} still pending.",
        "show_data_removed_from_the_device" => "Show data removed from the device.",
        "show" => "Show",
        "loading_shows" => "Loading shows…",
        "select_an_event" => "Select an event",
        "gate_works_locally" => "Gate works locally",
        "works_without_lte" => "Works without LTE",
        "download_a_secure_snapshot_before_opening_the" => {
            "Download a secure snapshot before opening the gates."
        }
        "prepare_offline" => "PREPARE OFFLINE",
        "sync" => "SYNC",
        "clear" => "CLEAR",
        "verifying" => "VERIFYING…",
        "scan_locally" => "SCAN LOCALLY",
        "open_camera" => "OPEN CAMERA",
        "durable_t1_ticket_qr_only" => "Durable t1 ticket QR only",
        "ticket_or_admission_pass_qr" => "Ticket or admission-pass QR",
        "qr_code_or_admission_pass_number" => "QR code or admission-pass number",
        "check" => "CHECK",
        "select_a_show_and_enter_the_fan" => "Select a show and enter the fan email.",
        "enter_the_admission_pass_public_reference" => "Enter the admission pass public reference.",
        "admission_pass_has_been_revoked" => "The admission pass has been revoked.",
        "select_a_show_2" => "Select a show",
        "sold" => "sold",
        "in_checkout" => "in checkout",
        "available_label" => "available",
        "refunds" => "refunds: {}",
        "admission_pass_number_is_a_safe_public" => {
            "The admission-pass number is a safe public identifier, e.g. VRY-... It is not a QR token or a private order token."
        }
        "issue_pass" => "ISSUE PASS",
        "admission_pass_number_e_g_vry" => "Admission-pass number, e.g. VRY-…",
        "revoke" => "REVOKE",
        "enter_the_code_and_sale_number" => "Enter the code and sale number.",
        "discount_code" => "Discount code",
        "sale_number" => "Sale number",
        "redeem_coupon" => "REDEEM COUPON",
        "usage" => "Usage {}/{}",
        "main_entrance" => "Main entrance",
        "enter_a_valid_start_date" => "Enter a valid start date.",
        "enter_a_valid_end_date" => "Enter a valid end date.",
        "limit_must_be_a_positive_number" => "The limit must be a positive number.",
        "campaign_end_must_be_after_its_start" => "The campaign end must be after its start.",
        "select_a_show_and_name_the_campaign" => "Select a show and name the campaign.",
        "qr_campaign_created" => "QR campaign created.",
        "loading_campaigns" => "Loading campaigns…",
        "point_campaign_name" => "Point / campaign name",
        "valid_from" => "Valid from",
        "valid_until" => "Valid until",
        "check_in_limit_optional" => "Check-in limit (optional)",
        "create_campaign" => "CREATE CAMPAIGN",
        "campaign_has_been_disabled" => "The campaign has been disabled.",
        "disable_campaign" => "DISABLE CAMPAIGN",
        "limit_v" => "limit {v}",
        "no_limit" => "no limit",
        "connection" => "Connection",
        "permissions" => "Permissions",
        "refresh_all_data" => "Refresh all data",
        "lock_panel" => "Lock panel",
        "remove_operator_profile" => "Remove operator profile",
        "operator_token_is_stored_in_an_encrypted" => {
            "The operator token is stored in an encrypted Stronghold vault. The WebView layer never reads it."
        }
        "refresh_2" => "Refresh",
        "cockpit_is_partially_available_unavailable" => {
            "Cockpit is partially available. Unavailable: {}."
        }
        "no_error_code" => "no error code",
        "attempt" => "attempt {}/{}",
        "retry_had_already_been_accepted" => "The retry had already been accepted.",
        "item_returned_to_the_queue" => "The item returned to the queue.",
        "language" => "Language",
        "app_language" => "App language",
        "changing_the_language_reloads_the_interface_your" => {
            "Changing the language reloads the interface. Your data and session remain unchanged."
        }
        "polish" => "Polish",
        "english" => "English",
        "failed_to_read_the_fan_profile" => "FAILED TO READ THE FAN PROFILE",
        "checking_your_signal" => "CHECKING YOUR SIGNAL",
        "your_profile_remains_untouched" => "Your profile remains untouched.",
        "app_will_not_continue_to_signup_or" => {
            "The app will not continue to signup or pairing until it confirms the encrypted vault state on this device."
        }
        "try_again" => "TRY AGAIN",
        "enter_the_email_used_to_join_signal" => "Enter the email used to join Signal.",
        "paste_the_code_or_full_link_or" => {
            "Paste the code or full link, or scan the QR from the message."
        }
        "create_a_local_pin_with_at_least" => "Create a local PIN with at least 4 characters.",
        "marketing_consent_is_required_to_join_signal" => {
            "Marketing consent is required to join Signal."
        }
        "could_not_save_the_city_message" => "Could not save the city: {message}",
        "select_a_city_or_enter_your_own" => "Select a city or enter your own.",
        "we_sent_a_secure_access_link_scan" => {
            "We sent a secure access link. Scan the QR or paste the code from the message."
        }
        "we_sent_a_confirmation_code_scan_the" => {
            "We sent a confirmation code. Scan the QR or paste the code from the message."
        }
        "new_message_was_not_sent_because_the" => {
            "A new message was not sent because the previous code is still valid. Use the previous message or try again in about {minutes} min."
        }
        "request_was_accepted_check_your_inbox_and" => {
            "The request was accepted. Check your inbox and spam; if the message is missing, try again later."
        }
        "enter_the_email_used_in_virya_signal" => "Enter the email used in Virya Signal.",
        "if_this_email_is_registered_in_virya" => {
            "If this email is registered in Virya Signal, we sent a fresh login link with a QR code. After opening it, set a new PIN for this device."
        }
        "qr_scanned_enter_your_email_and_local" => "QR scanned. Enter your email and local PIN.",
        "shows_tickets" => "Shows, tickets",
        "and_rewards" => "and rewards.",
        "join_in_3_steps" => "Join in 3 steps:",
        "how_to_join" => "How to join",
        "enter_your_email_and_city" => " Enter your email and city",
        "confirm_the_code_from_the_message" => " Confirm the code from the message",
        "discover_shows_near_you" => " Discover shows near you",
        "what_virya_signal_gives_you" => "What Virya Signal gives you",
        "shows_near_you" => " shows near you",
        "tickets_and_qr_codes_on_your_phone" => " tickets and QR codes on your phone",
        "rewards_for_simple_actions" => " rewards for simple actions",
        "get_started" => "GET STARTED",
        "i_have_a_code" => "I HAVE A CODE",
        "email" => "Email",
        "name_optional" => "Name (optional)",
        "fastest_scan_the_qr_from_the_email" => "Fastest: scan the QR from the email.",
        "you_can_also_paste_the_full_link" => {
            "You can also paste the full link or the 64-character code. The app will extract the correct token."
        }
        "email_link_or_code" => "Email link or code",
        "paste_a_link_or_code_or_use" => "Paste a link or code, or use QR",
        "scan_qr" => "SCAN QR",
        "or_hold_the_field_above_and_choose" => "or hold the field above and choose Paste",
        "local_pin" => "Local PIN",
        "pin_encrypts_your_profile_on_this_device" => {
            "The PIN encrypts your profile on this device only. It is never sent to CrowdRelay."
        }
        "confirm_and_enter" => "CONFIRM AND ENTER",
        "i_already_have_an_account_send_login" => "I ALREADY HAVE AN ACCOUNT — SEND LOGIN LINK",
        "no_message_check_spam_after_15_minutes" => {
            "No message? Check spam. After 15 minutes, return to GET STARTED and request another code."
        }
        "city" => "City",
        "e_g_bielawa" => "e.g. Bielawa",
        "province_region_optional" => "Province / region (optional)",
        "lower_silesia" => "Lower Silesia",
        "enter_your_city_manually_we_will_match" => {
            "Enter your city manually — we will match it to the Signal map."
        }
        "notify_me_about_nearby_shows" => "Notify me about nearby shows",
        "referral_code_optional" => "Referral code (optional)",
        "i_want_to_receive_information_about_virya" => {
            "I want to receive information about Virya shows, releases and rewards."
        }
        "join_signal" => "JOIN SIGNAL",
        "open_my_signal" => "OPEN MY SIGNAL",
        "i_forgot_my_pin_sign_in_again" => "I FORGOT MY PIN / SIGN IN AGAIN",
        "access_recovery" => "ACCESS RECOVERY",
        "create_a_new_pin" => "Create a new PIN",
        "enter_your_email_request_a_fresh_link" => {
            "Enter your email, request a fresh link, then scan the QR or paste the code."
        }
        "send_login_link" => "SEND LOGIN LINK",
        "paste_link_or_code" => "Paste link or code",
        "or_hold_the_field_and_choose_paste" => "or hold the field and choose Paste",
        "new_local_pin" => "New local PIN",
        "confirm_and_set_new_pin" => "CONFIRM AND SET NEW PIN",
        "back_to_pin_login" => "BACK TO PIN LOGIN",
        "code" => "Code: {}",
        "loading_signal" => "Loading Signal…",
        "entries" => "entries",
        "coupons" => "coupons",
        "draw" => "Draw {}",
        "proof" => "PROOF ↗",
        "merch" => "Merch",
        "products_and_bundles_use_the_same_inventory" => {
            "Products and bundles use the same inventory as the online store. Payment opens secure Stripe Checkout and the app never stores card data."
        }
        "store_is_temporarily_unavailable" => "The store is temporarily unavailable",
        "rest_of_signal_is_working_normally_try" => {
            "The rest of Signal is working normally. Try again in a moment."
        }
        "refresh_merch" => "REFRESH MERCH",
        "open_full_store" => "OPEN FULL STORE ↗",
        "bundles" => "BUNDLES",
        "bundles_from_the_online_store" => "Bundles from the online store",
        "up_to_30" => "UP TO −30%",
        "bundles_are_currently_unavailable_in_live_inventory" => {
            "Bundles are currently unavailable in live inventory."
        }
        "view_bundles" => "VIEW BUNDLES ↗",
        "low_stock" => "LOW STOCK",
        "available_status" => "AVAILABLE",
        "out_of_stock" => "OUT OF STOCK",
        "check_again" => "CHECK AGAIN",
        "buy_in_store" => "BUY IN STORE ↗",
        "bundles_load_independently_from_products" => "Bundles load independently from products.",
        "individual_products" => "INDIVIDUAL PRODUCTS",
        "choose_your_merch" => "Choose your merch",
        "pre_order" => "PRE-ORDER",
        "could_not_load_store_status" => "Could not load store status",
        "shows_tickets_and_profile_remain_available" => {
            "Shows, tickets and profile remain available."
        }
        "no_shows_in_the_calendar" => "No shows in the calendar",
        "new_events_will_appear_here_2" => "New events will appear here.",
        "show_saved_to_your_signal" => "Show saved to your Signal.",
        "saving" => "SAVING…",
        "saved" => "✓ SAVED",
        "interested" => "+ INTERESTED",
        "claimed" => "CLAIMED",
        "redeemed" => "REDEEMED",
        "buy_ticket" => "BUY TICKET",
        "back_back_to_shows" => "← BACK TO SHOWS",
        "could_not_check_ticket_sales" => "Could not check ticket sales",
        "no_virya_ticket_pool" => "No Virya ticket pool",
        "you_can_open_the_show_page_or" => {
            "You can open the show page or the organiser’s ticket sale."
        }
        "check_tickets" => "CHECK TICKETS ↗",
        "ticket_sales_will_open_soon" => "Ticket sales will open soon.",
        "online_sales_have_ended" => "Online sales have ended.",
        "this_ticket_pool_is_sold_out" => "This ticket pool is sold out.",
        "ticket_sales_are_temporarily_disabled" => "Ticket sales are temporarily disabled.",
        "this_show_is_not_currently_on_sale" => "This show is not currently on sale.",
        "tickets_are_not_available_right_now" => "Tickets are not available right now.",
        "select_tickets_places_will_be_reserved_while" => {
            "Select tickets. Places will be reserved while you complete payment."
        }
        "in_checkout_2" => "in checkout",
        "open_the_show_page" => "Open the show page",
        "if_the_organiser_runs_a_separate_ticket" => {
            "If the organiser runs a separate ticket sale, you will find it under this button."
        }
        "select_at_least_one_ticket" => "Select at least one ticket.",
        "order_saved_complete_the_secure_stripe_payment" => {
            "Order {} saved. Complete the secure Stripe payment."
        }
        "payment_opened_for_order" => "Payment opened for order {}.",
        "available" => "Available: {}",
        "ticket_quantity" => "Ticket quantity",
        "decrease_ticket_quantity" => "Decrease ticket quantity",
        "increase_ticket_quantity" => "Increase ticket quantity",
        "name_on_the_order_optional" => "Name on the order (optional)",
        "tickets_and_confirmation_will_be_sent_to" => "Tickets and confirmation will be sent to {}",
        "tickets_will_be_sent_to_the_fan" => "Tickets will be sent to the fan account email.",
        "invoice_full_form" => "INVOICE / FULL FORM ↗",
        "selected_tickets" => "Selected tickets",
        "gross_total" => "Gross total",
        "reserving" => "RESERVING…",
        "order_saved" => "ORDER SAVED",
        "continue_to_stripe_payment" => "CONTINUE TO STRIPE PAYMENT",
        "reopen_payment" => "REOPEN PAYMENT ↗",
        "card_details_never_reach_virya_signal_payment" => {
            "Card details never reach Virya Signal. Payment opens in secure Stripe Checkout."
        }
        "open_map_and_start" => "OPEN MAP AND START",
        "refresh_progress" => "Refresh progress",
        "open_area" => "OPEN AREA",
        "enter_the_order_id_and_private_token" => "Enter the order ID and private token.",
        "tickets_saved_to_the_wallet" => "Tickets saved to the wallet.",
        "paste_the_admission_pass_token" => "Paste the admission-pass token.",
        "admission_pass_assigned_to_this_device" => "Admission pass assigned to this device.",
        "show_entry_qr" => "SHOW ENTRY QR",
        "token_from_the_message" => "Token from the message",
        "claim_admission_pass" => "CLAIM ADMISSION PASS",
        "add_an_existing_order" => "Add an existing order",
        "order_uuid" => "Order UUID",
        "private_checkout_token" => "Private checkout token",
        "add_to_wallet" => "ADD TO WALLET",
        "we_resent_the_wallet_by_email" => "We resent the wallet by email.",
        "sending" => "Sending…",
        "resend_tickets_by_email" => "Resend tickets by email",
        "generating" => "GENERATING…",
        "hide_qr" => "HIDE QR",
        "show_qr" => "SHOW QR",
        "qr_unavailable" => "QR UNAVAILABLE",
        "qr_valid_until" => "QR valid until {}",
        "wallet_ticket_ready" => "Ready for entry",
        "wallet_ticket_used" => "Ticket already used",
        "wallet_ticket_used_at" => "Used {}",
        "wallet_ticket_revoked" => "Ticket revoked",
        "wallet_ticket_expired" => "Ticket expired",
        "wallet_ticket_not_claimed" => "Ticket is not active yet",
        "valid_until_2" => "valid until {}",
        "my_profile" => "MY PROFILE",
        "signal_settings" => "Signal settings",
        "virya_fan" => "Virya fan",
        "orders" => "orders",
        "admission_passes" => "admission passes",
        "refreshing_2" => "Refreshing…",
        "refresh_data" => "Refresh data",
        "lock_app" => "Lock app",
        "remove_profile_and_tickets_from_device" => "Remove profile and tickets from device",
        "fan_session_admission_pass_and_private_wallet" => {
            "The fan session, admission pass and private wallet tokens are stored in a separate encrypted Stronghold vault."
        }
        "feedback_must_contain_between_8_and_2000" => {
            "Feedback must contain between 8 and 2000 characters."
        }
        "feedback_was_sent_anonymously_thank_you" => {
            "Feedback was accepted anonymously. If you are offline, it will be sent automatically when the connection returns."
        }
        "anonymous_feedback" => "ANONYMOUS FEEDBACK",
        "tell_us_what_to_improve" => "Tell us what to improve",
        "app_sends_only_the_category_and_message" => {
            "The app sends only the category and message — no email, name, session token or profile identifier. Hosting may retain standard technical connection logs."
        }
        "category" => "Category",
        "idea" => "Idea",
        "bug_label" => "Bug",
        "shows_and_tickets" => "Shows and tickets",
        "other" => "Other",
        "message" => "Message",
        "tell_us_directly_what_is_broken_or" => "Tell us directly what is broken or missing…",
        "sending_2" => "SENDING…",
        "send_anonymously" => "SEND ANONYMOUSLY",
        "loading" => "Loading",
        "feedback_was_sent" => "feedback was sent",
        "could_not_refresh_orders_the_remaining_tickets" => {
            "Could not refresh {} orders. The remaining tickets are available."
        }
        "details_coming_soon" => "Details coming soon",
        "venue_coming_soon" => "venue coming soon",
        "scanner_returned_no_code" => "The scanner returned no code.",
        "server_response_has_an_unexpected_format" => {
            "The server response has an unexpected format."
        }
        "response_decoding_error_raw" => "Response decoding error: {raw}",
        "unknown_application_error" => "Unknown application error",
        "pair" => "Pair",
        "device" => "device.",
        "no_retyping_the_api_role_or_long" => "No retyping the API, role or long secret.",
        "pair_2" => "PAIR",
        "operator_profile_is_encrypted_locally" => "The operator profile is encrypted locally.",
        "today_under_control" => "Today under control",
        "next_show" => "NEXT SHOW",
        "upcoming" => "Upcoming",
        "community_and_growth" => "Community and growth",
        "combined_signal_overview_without_fans_personal_data" => {
            "A combined Signal overview without fans’ personal data."
        }
        "database_health" => "DATABASE HEALTH",
        "scan_entry" => "Scan entry",
        "tickets_and_admission_passes" => "Tickets and admission passes",
        "gross_revenue" => "GROSS REVENUE",
        "recent_orders" => "Recent orders",
        "manual_admission_pass" => "Manual admission pass",
        "redeem_a_discount" => "Redeem a discount",
        "fan_coupon_controlled_use" => "fan coupon / controlled use",
        "coupon_redeemed" => "COUPON REDEEMED",
        "qr_campaigns" => "QR campaigns",
        "active_and_historical" => "Active and historical",
        "queues_and_deliveries" => "Queues and deliveries",
        "dead_deliveries" => "Dead deliveries",
        "dead_outbox" => "Dead outbox",
        "no_dead_entries_the_delivery_pipeline_is" => {
            "No dead entries. The delivery pipeline is clean."
        }
        "your_profile_and_tickets_are_encrypted_on" => {
            "Your profile and tickets are encrypted on this device."
        }
        "your_impact" => "YOUR IMPACT",
        "confirmed_referrals" => "confirmed referrals",
        "your_coupons" => "Your coupons",
        "rewards" => "Rewards",
        "active_draws" => "Active draws",
        "entries_2" => "ENTRIES",
        "where_we_play" => "WHERE WE PLAY",
        "find_a_point_in_your_city" => "Find a point in your city",
        "open_the_map_choose_an_active_point" => {
            "Open the map, choose an active point and go there. You do not need to collect everything or travel across the country."
        }
        "connect_your_browser_wallet_to_your_area" => {
            "Connect your browser wallet to your AREA account to keep all progress."
        }
        "collection_progress" => "COLLECTION PROGRESS",
        "discovered_artifacts" => "Discovered artifacts",
        "map_shows_active_points_and_gets_you" => {
            "The map shows active points and gets you started. The exact location is revealed in the game only when needed."
        }
        "area_is_temporarily_unavailable" => "AREA is temporarily unavailable",
        "refresh_the_data_or_open_the_full" => "Refresh the data or open the full game.",
        "tickets_and_entry" => "Tickets and entry",
        "virya_admission_pass" => "VIRYA ADMISSION PASS",
        "did_you_win_an_admission_pass" => "DID YOU WIN AN ADMISSION PASS?",
        "assign_it_to_your_phone" => "Assign it to your phone",
        "ticket_wallet" => "Ticket wallet",
        "individual_products_2" => "Individual products",
        "bundles_2" => "Bundles",
        "retry" => "RETRY",
        "connecting_2" => "CONNECTING",
        "online" => "ONLINE",
        "offline_on" => "OFFLINE ON",
        "offline_off" => "OFFLINE OFF",
        "active_status" => "ACTIVE",
        "closed" => "CLOSED",
        "owner" => "OWNER",
        "staff" => "STAFF",
        "staff_session_expired_pair_again" => {
            "This staff device session has expired. Pair the device again before online staff operations."
        }
        "staff_session_expires_soon_pair_again" => {
            "This staff device session expires within 24 hours. Pair the device again before the next show."
        }
        "device_label" => "DEVICE",
        "mobile_wallet" => "MOBILE WALLET",
        "virya_control" => "VIRYA CONTROL",
        "virya_signal" => "VIRYA SIGNAL",
        "virya_store" => "VIRYA STORE",
        "virya_area" => "VIRYA AREA",
        "virya_tickets" => "VIRYA // TICKETS",
        "tickets_pending_conflicts" => "{} tickets · {} pending · {} conflicts",
        "offline_show_mode_status" => "Offline show mode status",
        "eligible_tickets" => "eligible",
        "pending_scans" => "pending",
        "synced_scans" => "synced",
        "scan_conflicts" => "conflicts",
        "check_ins_2" => "{} check-ins",
        "attempt_2" => "{} · attempt {}/{}",
        "my_signal" => "My Signal",
        "pending_2" => "pending",
        "message_order_is_saved_use_the_reopen" => {
            "{message} Order {} is saved — use the reopen payment button."
        }
        "reward_credits_credits" => "{reward_credits} credits",
        "live_count_active_points" => "{live_count} active points",
        "voucher_count_rewards" => "{voucher_count} rewards",
        "community_percent_community" => "{community_percent}% community",
        "sent" => "sent",
        "revoked" => "revoked",
        "jan" => "JAN",
        "feb" => "FEB",
        "mar" => "MAR",
        "apr" => "APR",
        "may" => "MAY",
        "jun" => "JUN",
        "jul" => "JUL",
        "aug" => "AUG",
        "sep" => "SEP",
        "oct" => "OCT",
        "nov" => "NOV",
        "dec" => "DEC",
        "text" => "---",
        "active_2" => "{} active",
        "virya_merch_bundle" => "{} — Virya merch bundle",
        "virya_merch" => "{} — Virya merch",
        "virya_show" => "{} — Virya show",
        "native_app_bridge_is_unavailable" => "The native app bridge is unavailable.",
        "operation_command_timed_out" => "Operation {command} timed out.",
        "camera_permission_module_is_unavailable_in_this" => {
            "The camera permission module is unavailable in this app version."
        }
        "camera_access_is_denied_enable_camera_for" => {
            "Camera access is denied. Enable Camera for Virya Signal in the app settings."
        }
        "qr_code_scanner" => "QR code scanner",
        "scan_qr_code_2" => "SCAN QR CODE",
        "place_the_code_inside_the_frame" => "Place the code inside the frame",
        "back_cancel_scanning" => "← CANCEL SCANNING",
        "closing" => "CLOSING…",
        "scanner_is_available_only_in_the_ios" => {
            "The scanner is available only in the iOS/Android app."
        }
        "type" => "Type",
        "time" => "Time",
        "operation" => "Operation",
        "path" => "Path",
        "previous_launch_ended_with_an_error" => "The previous launch ended with an error",
        "app_caught_an_error" => "The app caught an error",
        "we_do_not_hide_failures_copy_the" => {
            "We do not hide failures. Copy the report and send it with a note about what you tapped."
        }
        "copy_report" => "COPY REPORT",
        "restart_app" => "RESTART APP",
        "close" => "CLOSE",
        "report_copied" => "Report copied.",
        "press_and_hold_the_report_text_and" => {
            "Press and hold the report text and copy it manually."
        }
        "previous_launch_interrupted_operation_command" => {
            "The previous launch interrupted operation {command}."
        }
        "previous_launch_ended_without_a_clean_shutdown" => {
            "The previous launch ended without a clean shutdown."
        }
        "virya_signal_diagnostics" => "VIRYA SIGNAL / DIAGNOSTICS",
        "native_error_not_configured" => "The device profile is not configured",
        "native_error_invalid_pin" => "Invalid PIN",
        "native_error_locked" => "The session is locked",
        "native_error_unauthorized" => {
            "The device token is invalid or lacks the required permissions"
        }
        "native_error_forbidden" => "This operation requires the owner role",
        "native_error_conflict" => "Conflict",
        "native_error_not_found" => "No data found",
        "native_error_crowdrelay" => "CrowdRelay",
        "native_error_network" => "Network error",
        "native_error_url" => "Invalid URL",
        "native_error_data" => "Data error",
        "native_error_file" => "File error",
        "native_error_vault" => "Vault storage error",
        "native_error_background_task" => "Internal task error",
        "native_pin_4_128" => "The PIN must contain 4–128 characters",
        "native_damaged_device_profile" => "The device profile is corrupted",
        "native_invalid_device_name" => "Invalid device name",
        "native_invalid_device_token" => "Invalid device token",
        "native_complete_fan_data" => "Complete the fan details correctly",
        "native_paste_valid_code" => "Paste a valid code or link, or scan the QR code",
        "native_invalid_email_or_token" => "Invalid email address or token",
        "native_invalid_pass_data" => "Invalid pass data",
        "native_invalid_qr_campaign_data" => "Invalid QR campaign data",
        "native_operator_pin_4_6" => "The operator PIN must contain 4–6 digits",
        "native_pin_min_4" => "The PIN must contain at least 4 characters",
        "native_api_must_use_https" => "The API must use HTTPS",
        "native_invalid_api_base_url" => "Invalid API base URL",
        "native_backend_update_required" => {
            "The server needs an update before this feature can be used."
        }
        "native_invalid_label" => "Invalid {label}",
        "native_public_cache_too_large" => "The local public-data cache is too large",
        "native_missing_events_cache" => "The backend confirmed a non-existent event cache",
        "native_missing_cities_cache" => "The backend confirmed a non-existent city cache",
        "native_missing_merch_cache" => "The backend confirmed a non-existent merch cache",
        "native_invalid_staff_password" => "Invalid staff password.",
        "native_staff_rate_limited" => "Too many sign-in attempts. Try again in several minutes.",
        "native_staff_verification_unavailable" => "Staff verification is temporarily unavailable",
        "native_staff_verification_failed" => "Staff access could not be verified",
        "native_invalid_store_url" => "Invalid store address",
        "native_bundle_catalog_too_large" => "The bundle catalog is too large",
        "native_invalid_merch_bundle" => "Invalid merch bundle",
        "native_bundle_too_many_items" => "The merch bundle contains too many items",
        "native_invalid_bundle_offer" => "Invalid bundle offer",
        "native_invalid_bundle_variant" => "Invalid bundle variant",
        "native_choose_feedback_category" => "Choose a feedback category",
        "native_feedback_content_label" => "feedback content",
        "native_feedback_failed" => "The feedback could not be submitted",
        "native_response_too_large" => "The CrowdRelay response is too large",
        "native_operation_rejected" => "CrowdRelay rejected the operation",
        "native_production_api_https" => "The production API URL must use HTTPS",
        "native_invalid_identifier" => "Invalid identifier",
        "native_invalid_order_id" => "Invalid order identifier",
        "native_invalid_qr_code" => "Invalid QR code",
        "native_ticket_offer_invalid" => "The server returned an invalid ticket offer",
        "native_ticket_pool_invalid" => "The server returned an invalid ticket pool",
        "native_event_id_invalid" => "Invalid event identifier",
        "native_buyer_name_too_long" => "The name is too long",
        "native_choose_tickets" => "Choose tickets",
        "native_ticket_selection_invalid" => "Invalid ticket selection",
        "native_too_many_tickets" => "Too many tickets selected",
        "native_payment_url_invalid" => "The server returned an invalid payment address",
        "native_order_invalid" => "The server returned an invalid order",
        "native_order_incomplete" => "The server returned incomplete order data",
        "native_admission_token_label" => "admission token",
        "native_order_token_label" => "order token",
        "native_qr_token_label" => "QR token",
        "native_coupon_code_label" => "coupon code",
        "native_sale_number_label" => "sale number",
        "native_admission_session_missing" => "The backend did not return an admission session",
        "native_claim_pass_first" => "Claim the pass first",
        "native_code_already_used" => {
            "This code has already been used. Return to START and request a new message."
        }
        "native_code_invalid_or_expired" => {
            "The code is invalid or has expired. Request a new message."
        }
        "native_fan_session_missing" => {
            "The backend confirmed the code but did not return a fan session"
        }
        "native_area_wallet_id_invalid" => "Invalid AREA wallet identifier",
        "native_queue_type_invalid" => "Invalid queue type",
        "native_snapshot_time_invalid" => "The snapshot has an invalid timestamp",
        "native_offline_t1_only" => "Offline mode supports only durable t1 tickets",
        "native_ticket_qr_invalid" => "Invalid ticket QR code",
        "native_event_invalid" => "Invalid event",
        "native_snapshot_event_mismatch" => "CrowdRelay returned a snapshot for a different event",
        "native_snapshot_expired" => "The event snapshot is invalid or has expired",
        "native_snapshot_too_large" => "The snapshot exceeds the safe limit of 10,000 entries",
        "native_snapshot_integrity_failed" => "The event snapshot failed its integrity check",
        "native_qr_too_long" => "The QR code is too long",
        "native_snapshot_refresh_required" => {
            "The event snapshot has expired. Connect to the network and download a new one"
        }
        "native_ticket_not_in_snapshot" => "The ticket is not present in the signed snapshot",
        "native_scan_queue_full" => "The local scan queue is full",
        "native_no_prepared_event" => "No event has been prepared",
        "native_link_too_long" => "The link is too long",
        "native_https_links_only" => "Only secure HTTPS links can be opened",
        "native_open_link_failed" => "The link could not be opened: {error}",
        "native_city_name_invalid" => "Invalid city name",
        "native_enter_valid_staff_password" => "Enter a valid staff password.",
        "native_pairing_code_expired" => "The pairing code has expired or is invalid",
        "native_pairing_code_invalid" => "Invalid pairing code",
        "native_pairing_code_empty" => "The pairing code contains no data",
        "native_enter_valid_email" => "Enter a valid email address",
        "native_wallet_limit" => "The wallet can contain at most {max} orders",
        "native_wrong_order_wallet" => "The backend returned a wallet for a different order",
        "native_ticket_reference_invalid" => "Invalid ticket reference",
        "native_ticket_not_on_device" => "The ticket was not found on this device",
        "native_qr_token_missing" => "The backend response does not contain a QR token",
        "native_qr_token_invalid" => "Invalid QR token",
        "native_qr_generation_failed" => "The QR code could not be generated",
        "boot_starting" => "STARTING VIRYA SIGNAL",
        "boot_loading_secure_profile" => "Loading the secure device profile…",
        "boot_taking_longer" => "This is taking longer than usual",
        "boot_still_starting" => "The app is still starting. You can wait or try again.",
        "boot_retry" => "TRY AGAIN",
        "boot_diagnostics" => "DIAGNOSTICS",
        "boot_launch_failed" => "The app could not be started",
        "boot_reload_help" => {
            "Restart the app. If the problem returns, open diagnostics and send the report."
        }
        "boot_previous_terminated" => "The previous launch disappeared during phase {phase}.",
        "boot_phase_wasm_loading" => "LOADING APP ENGINE",
        "boot_phase_wasm_entered" => "STARTING INTERFACE",
        "boot_phase_wasm_initialized" => "FINISHING STARTUP",
        "boot_unknown_error" => "Unknown startup error",
        "boot_start_stopped" => "APP STARTUP STOPPED",
        "boot_module_not_started" => "THE APP MODULE DID NOT START",
        "boot_engine_load_failed" => "FAILED TO LOAD THE APP ENGINE",
        "boot_engine_no_interface" => "THE ENGINE DID NOT START THE INTERFACE",
        "boot_interface_incomplete" => "THE INTERFACE DID NOT FINISH STARTING",
        "boot_start_incomplete" => "STARTUP DID NOT COMPLETE",
        "boot_stage_retry_detail" => {
            "Phase: {phase}. Retry will perform one clean WebView restart."
        }
        "boot_retry_failed" => "RETRY DID NOT HELP",
        "boot_retry_blocked_detail" => {
            "Phase: {phase}. Save this message; the app will not enter another restart loop."
        }
        "boot_almost_ready" => "ALMOST READY — FINISHING STARTUP",
        "boot_initial_status" => "STARTING SIGNAL",
        "boot_retry_button" => "RETRY STARTUP",
        "boot_noscript" => "Virya Signal requires JavaScript/WASM.",
        "network_offline_cached" => "OFFLINE — CACHED DATA STAYS AVAILABLE",
        "network_restored" => "CONNECTION RESTORED",
        "native_bundle_name_label" => "bundle name",
        "native_bundle_description_label" => "bundle description",
        "native_bundle_item_label" => "bundle item",
        "native_image_url_label" => "image address",
        "native_store_url_label" => "store address",
        "native_bundle_variant_label" => "bundle variant",
        "native_prepare_offline_event_first" => "Prepare the event for offline mode first",
        "location_module_is_unavailable_in_this_app" => {
            "The location module is unavailable in this app version."
        }
        "location_access_is_denied_enable_location_for" => {
            "Location access is denied. Enable Location for Virya Signal in the app settings."
        }
        "could_not_read_a_fresh_location_move" => {
            "Unfortunately, this is not the correct location. Keep looking!"
        }
        "native_area_claim_invalid" => "The AREA point verification data is invalid.",
        "native_area_drop_inactive" => "This AREA point is not active now.",
        "native_area_challenge_invalid" => {
            "The location attempt expired. Start verification again."
        }
        "native_area_rate_limited" => "Too many attempts. Wait a few minutes and try again.",
        "native_area_not_enough_samples" => {
            "Not enough fresh location samples were collected. Stay in place briefly and try again."
        }
        "native_area_low_accuracy" => "Location accuracy is too low. Move outdoors and try again.",
        "native_area_outside_zone" => "You are outside the active AREA point zone.",
        "native_area_drop_full" => "This AREA point has reached its claim limit.",
        "native_area_claim_conflict" => "The claim was already processed. Refresh AREA progress.",
        "native_area_temporary" => {
            "AREA verification is temporarily unavailable. Try again in a moment."
        }
        "area_in_the_app" => "AREA IN THE APP",
        "choose_an_active_point_and_follow_the" => {
            "Choose an active point, check the direction and confirm your win without leaving the app."
        }
        "active_area_point" => "Active AREA point",
        "inactive_area_point" => "Point awaiting activation",
        "claimed_area_point" => "Point already discovered",
        "locate_nearest_point" => "LOCATE NEAREST POINT",
        "locating_you" => "LOCATING…",
        "nearest_active_point_is_city" => "Nearest active point: {city}.",
        "you_are_about_distance_from_city" => "You are about {distance} from the point in {city}.",
        "no_active_area_points_now" => {
            "There are no active AREA points now. The map still shows cities for future activations."
        }
        "open_route_start" => "OPEN ROUTE START",
        "verify_location_and_win" => "VERIFY LOCATION AND WIN",
        "verifying_location" => "VERIFYING LOCATION…",
        "area_location_privacy" => {
            "Location is used only during this attempt. The app sends a few fresh samples for verification and does not store your route."
        }
        "area_point_won" => "POINT DISCOVERED",
        "area_point_already_won" => "YOU ALREADY HAVE THIS POINT",
        "area_reward_added" => {
            "“{track}” and {credits} reward credit were added to your collection."
        }
        "area_reward_already_present" => "“{track}” is already in your collection.",
        "select_an_active_point_first" => "Select an active AREA point first.",
        "location_accuracy_value" => "Accuracy: ±{accuracy} m",
        "approximate_distance_meters" => "{distance} m",
        "approximate_distance_kilometers" => "{distance} km",
        "open_full_area_game" => "OPEN FULL AREA GAME",
        "fan_app_unlock_pin" => "Fan profile unlock PIN",
        "enter_the_pin_created_for_this_fan" => {
            "Enter the PIN created when this fan profile was set up. It is not the QR code or your phone PIN."
        }
        "create_fan_unlock_pin" => "Create a fan profile PIN",
        "enter_4_6_digits_for_this_fan_profile" => {
            "Enter 4–6 digits, for example 2580. This PIN unlocks only the fan profile in Virya Signal."
        }
        "this_show_has_no_ticket_pool" => "This show has no ticket pool.",
        "ticket_pool_status_loading" => "CHECKING TICKET POOL…",
        "ticket_pool_temporarily_unavailable" => "The ticket pool is temporarily unavailable.",
        "could_not_save_city_message" => "Could not save the city: {message}.",
        "new_message_not_sent_previous_code_still_valid_minutes" => {
            "A new message was not sent because the previous code is still valid. Try again in about {minutes} min."
        }
        "could_not_refresh_orders_count_other_tickets_remain_available" => {
            "Could not refresh {count} orders. Your other tickets remain available."
        }
        "area_city_wroclaw" => "Wroclaw",
        "area_city_poznan" => "Poznan",
        "area_city_gdansk" => "Gdansk",
        "area_city_warsaw" => "Warsaw",
        "area_city_katowice" => "Katowice",
        "area_city_krakow" => "Krakow",
        "area_city_lodz" => "Lodz",
        "area_city_szczecin" => "Szczecin",
        "area_city_lublin" => "Lublin",
        "area_city_rzeszow" => "Rzeszow",
        "area_city_bialystok" => "Bialystok",
        "area_city_torun" => "Torun",
        "area_region_lower_silesia" => "Lower Silesia",
        "area_region_greater_poland" => "Greater Poland",
        "area_region_pomerania" => "Pomerania",
        "area_region_masovia" => "Masovia",
        "area_region_silesia" => "Silesia",
        "area_region_lesser_poland" => "Lesser Poland",
        "area_region_lodz" => "Lodz region",
        "area_region_west_pomerania" => "West Pomerania",
        "area_region_lublin" => "Lublin region",
        "area_region_subcarpathia" => "Subcarpathia",
        "area_region_podlasie" => "Podlasie",
        "area_region_kuyavia_pomerania" => "Kuyavia-Pomerania",
        "area_clue_wroclaw" => "A signal is forming somewhere between concrete, water and noise.",
        "area_clue_poznan" => "Follow the gold signal. Leave the obvious route behind.",
        "area_clue_gdansk" => "Look for the echo where steel meets salt.",
        "area_clue_warsaw" => "The loudest city hides its quietest transmission.",
        "area_clue_katowice" => "An industrial pulse is waiting below the surface.",
        "area_clue_krakow" => "Old stone. New noise. One line locked inside.",
        "area_clue_lodz" => "Follow the thread through brick, rails and reinvention.",
        "area_clue_szczecin" => "The signal drifts inland from water shaped like a maze.",
        "area_clue_lublin" => "Listen where old gates carry a new frequency.",
        "area_clue_rzeszow" => "A southern pulse hides between motion and open sky.",
        "area_clue_bialystok" => "At the forest's edge, the quiet signal travels furthest.",
        "area_clue_torun" => "Look up, then follow the orbit back to the street.",
        "audience_intelligence" => "Audience Intelligence",
        "fan_360_summary" => "Fan 360 · aggregates",
        "ticket_buyers" => "ticket buyers",
        "concert_attendees" => "concert attendees",
        "synesthesia_participants" => "Synesthesia",
        "qualified_referrals" => "qualified referrals",
        "ticket_revenue" => "ticket revenue",
        "after_refunds" => "after refunds",
        "paid_orders_count" => "{0} paid orders",
        "direction_to_point" => "{arrow} {direction}",
        "direction_north" => "north",
        "direction_northeast" => "north-east",
        "direction_east" => "east",
        "direction_southeast" => "south-east",
        "direction_south" => "south",
        "direction_southwest" => "south-west",
        "direction_west" => "west",
        "direction_northwest" => "north-west",
        "signal_live_now" => "SIGNAL LIVE NOW",
        "signal_afterglow" => "AFTER THE SIGNAL",
        "open_wallet_now" => "OPEN TICKET / PASS",
        "open_live_signal" => "OPEN SHOW",
        "share_post_show_feedback" => "LEAVE A POST-SHOW ECHO",
        "get_ticket_now" => "GET A TICKET",
        "follow_this_signal" => "FOLLOW THIS SIGNAL",
        "signal_live_note" => "You are in the show window — the essentials are one tap away.",
        "signal_afterglow_note" => {
            "The show just ended. Leave a short anonymous echo while it is still fresh."
        }
        "unsupported_signal_snapshot_version" => {
            "This Signal snapshot uses unsupported schema version {}. Please update the app."
        }
        "unsupported_staff_snapshot_version" => {
            "This staff snapshot uses unsupported schema version {}. Please update the app."
        }
        "signal_snapshot_updated" => "Updated {}",
        "synesthesia_completed_in_minutes" => "Completed in about {} min",
        "reward_entry_confirmed" => "Reward entry confirmed",
        "doors_open_at" => "Doors: {}",
        "event_ends_at" => "Ends: {}",
        "entry_ready" => "Entry ready",
        "following_event" => "Following",
        "tickets_on_sale" => "Tickets on sale",
        "paid_orders" => "paid orders",
        "pending_referrals" => "pending referrals",
        "passes_issued" => "passes issued",
        "could_not_refresh_orders_cached_orders_available" => {
            "Could not refresh {} orders; {} offline copies remain available."
        }
        "autopilot_control" => "Autonomous operations",
        "autopilot_runtime_disabled" => {
            "Autopilot runtime is globally disabled. Policies can be prepared, but no decision will execute."
        }
        "autopilot_nothing_needs_you" => {
            "Nothing needs your decision — ViryaOS is handling current operations."
        }
        "autopilot_needs_you" => "Needs you",
        "autopilot_recent_actions" => "Recent actions",
        "autopilot_measured_effects" => "Measured effect",
        "autopilot_effect_improved" => "improved",
        "autopilot_effect_neutral" => "no material change",
        "autopilot_effect_worsened" => "worsened",
        "autopilot_actions_24h" => "executed 24h",
        "autopilot_queue" => "queued",
        "autopilot_failed_24h" => "failed 24h",
        "autopilot_authority" => "Autonomy level",
        "autopilot_financial_guardrails" => "Financial guardrails",
        "autopilot_off" => "OFF",
        "autopilot_observe" => "OBSERVE",
        "autopilot_recommend" => "RECOMMEND",
        "autopilot_approval" => "APPROVAL",
        "autopilot_auto" => "AUTO",
        "autopilot_guarded" => "AUTO GUARD ACTIVE",
        "autopilot_expires" => "approval expires",
        "autopilot_executor_confirmed" => "executor confirmed",
        "autopilot_executor_failed" => "executor failed",
        "autopilot_release_ledger" => "Release ledger",
        "autopilot_release_drift" => "DRIFT",
        "autopilot_release_sync" => "SYNC",
        "autopilot_n8n_executors" => "n8n executors",
        "autopilot_executor_guards" => "executor guards",
        "autopilot_release_missing" => "missing",
        "autopilot_release_stale" => "STALE",
        "autopilot_release_production" => "production",
        "autopilot_rum_24h" => "Real-user performance · 24h",
        "autopilot_samples" => "samples",
        "autopilot_assign" => "ASSIGN",
        "autopilot_assign_to" => "Assign task",
        "autopilot_approve" => "APPROVE",
        "autopilot_cancel" => "CANCEL",
        "autopilot_chief" => "Operations chief",
        "autopilot_time_saved" => "team time saved",
        "autopilot_improved_7d" => "improved 7d",
        "autopilot_deadline_radar" => "Deadline radar",
        "autopilot_attention_approval" => "Approval expires",
        "autopilot_attention_opportunity" => "Opportunity deadline",
        "autopilot_attention_funding" => "Funding deadline",
        "autopilot_urgency_overdue" => "OVERDUE",
        "autopilot_urgency_critical" => "URGENT",
        "autopilot_urgency_today" => "TODAY",
        "autopilot_urgency_soon" => "SOON",
        "autopilot_urgency_upcoming" => "UPCOMING",
        "autopilot_opportunities" => "Top opportunities",
        "autopilot_show_tasks" => "Show tasks to close",
        "autopilot_manual_steps" => "Manual: {}",
        "autopilot_beacon_discovery_detail" => "Beacon · find {} local lighthouses",
        "autopilot_invalid_action" => "Invalid Autopilot action",
        "autopilot_funding_package_detail" => "Prepare funding application package",
        "autopilot_funding_submit_detail" => "Submit prepared funding application",
        "database_runtime" => "PostgreSQL runtime",
        "async_io" => "async I/O",
        "area_runtime" => "AREA authority",
        "area_credits" => "credits",
        "area_vouchers" => "vouchers",
        "area_ticket_rewards" => "ticket rewards",
        "area_legacy_imported" => "legacy imported",
        "stale_voucher_leases" => "stale voucher leases",
        "stale_ticket_leases" => "stale ticket leases",
        "push_notifications" => "Push notifications",
        "push_notifications_hint" => {
            "Nearby shows, tickets and important Signal messages can appear directly on this phone."
        }
        "push_notifications_on" => "Notifications are active on this device.",
        "push_notifications_off" => "Notifications are disabled on this device.",
        "push_notifications_blocked" => {
            "Android is blocking notifications. Enable them for Virya Signal in your phone settings."
        }
        "push_notifications_waiting_backend" => "The push channel is not live in VIRYA OS yet.",
        "push_notifications_degraded" => {
            "Push synchronization could not be confirmed. Retry without changing the setting."
        }
        "enable_push_notifications" => "ENABLE NOTIFICATIONS",
        "disable_push_notifications" => "DISABLE NOTIFICATIONS",
        "open_notification_settings" => "OPEN SETTINGS",
        "syncing_push_notifications" => "SYNCING…",
        "carry_the_signal" => "CARRY THE SIGNAL",
        "invite_real_metalheads" => "Activate people who might genuinely care",
        "invite_one_to_three_people_you_really_think_would_care" => {
            "Invite 1–3 people who actually listen to heavy music or are part of the scene. No spam — trust is the point."
        }
        "virya_signal_share_copy" => {
            "VIRYA is building Signal for people in the scene. If this feels like your thing, or someone in your circle would care, enter here:"
        }
        "share_signal" => "SHARE SIGNAL",
        "signal_shared" => "Signal carried forward.",
        "signal_link_copied" => "Signal link copied.",
        "show_operations" => "Show operations",
        "gig_checklist" => "GIG CHECKLIST",
        "gig_checklist_hint" => {
            "One shared checklist for the whole staff. Ticking an item here immediately syncs with the staff panel on virya.music."
        }
        "team_push_notifications" => "Staff reminders",
        "team_push_notifications_hint" => {
            "Signal will remind the team 7 and 2 days before the show."
        }
        "team_push_active" => "Checklist notifications are active on this device.",
        "team_push_inactive" => {
            "Enable notifications to get reminders 7 and 2 days before the show."
        }
        "team_push_status_unknown" => "Notification status is not available yet.",
        "enable_notifications" => "ENABLE NOTIFICATIONS",
        "checklist_progress" => "Done: {0}/{1}",
        "checklist_section_show_files" => "SET / FILES",
        "checklist_section_gear" => "GEAR",
        "checklist_section_media" => "MEDIA",
        "checklist_section_logistics" => "LOGISTICS",
        "checklist_section_gate" => "GATE / OFFLINE",
        "checklist_section_post_show" => "POST-SHOW",
        "checklist_laptop_charged_packed" => "Charge and pack the laptop",
        "checklist_setlist_ready" => "Prepare and verify the setlist",
        "checklist_show_files_backup_ready" => {
            "Make an offline backup of the set / playback / show files"
        }
        "checklist_merch_packed" => "Pack merch and everything needed for sales",
        "checklist_rack_cables_instruments_packed" => {
            "Pack the rack, personal cables and instruments"
        }
        "checklist_instrument_spares_packed" => "Pack spare strings, picks and required batteries",
        "checklist_stage_outfit_packed" => "Pack the stage outfit",
        "checklist_wireless_checked" => "Check wireless systems, frequencies and batteries",
        "checklist_power_and_chargers_packed" => {
            "Pack power supplies, chargers and required extension leads"
        }
        "checklist_camera_handoff_ready" => {
            "Give Madzia a working, charged high-quality camera + storage media"
        }
        "checklist_venue_schedule_confirmed" => {
            "Confirm address, parking, load-in, soundcheck and set time"
        }
        "checklist_tech_rider_confirmed" => {
            "Confirm rider / backline / technical requirements with the organizer"
        }
        "checklist_staff_assigned" => "Confirm show staff and responsibilities",
        "checklist_guestlist_checked" => "Check guest list / passes / organizer contacts",
        "checklist_offline_snapshot_ready" => "Prepare an offline snapshot of gate-critical data",
        "checklist_gate_device_charged" => "Charge the device used at the gate / merch desk",
        "checklist_backup_device_ready" => "Prepare a backup device in case of failure",
        "checklist_network_tested" => "Check internet / hotspot plan and offline mode",
        "checklist_post_show_reconciliation" => "Reconcile merch / tickets / cash after the show",
        "checklist_post_show_report" => "Record show result, media and improvements after the show",
        "checklist_unknown_item" => "Checklist item",
        "latarnik_vault_failed" => "FAILED TO READ THE LATARNIK VAULT",
        "latarnik_vault_checking" => "CHECKING LATARNIK VAULT",
        "latarnik_native_label" => "VIRYA SIGNAL · LATARNIK",
        "latarnik_private_network" => "A private network for people in the scene.",
        "latarnik_access_pitch" => {
            "Local shows, ready-to-use press assets, quick accreditation requests and direct contact — without newsletter noise."
        }
        "latarnik_invite_received" => "INVITATION RECEIVED",
        "latarnik_invite_received_hint" => {
            "The invitation link was securely picked up by the app. Set a PIN and activate Latarnik."
        }
        "latarnik_pin_create" => "Create a Latarnik PIN",
        "latarnik_pin_hint" => {
            "4–6 digits. This PIN encrypts a separate Latarnik profile on this device."
        }
        "latarnik_topic_shows" => "Shows",
        "latarnik_topic_press" => "Press assets",
        "latarnik_topic_releases" => "Releases",
        "latarnik_topic_interviews" => "Interviews",
        "latarnik_topic_accreditation" => "Accreditation",
        "latarnik_activate_invite" => "ACTIVATE INVITATION",
        "latarnik_scan_invite" => "SCAN INVITATION QR",
        "latarnik_scan_hint" => "The same one-time link works in QR and in the browser.",
        "latarnik_invite_link" => "Invitation link or code",
        "latarnik_paste_invite" => "Paste the email link or one-time code",
        "latarnik_activate_pasted" => "ACTIVATE PASTED INVITATION",
        "latarnik_vault_locked" => "Your Latarnik profile is encrypted on this device.",
        "latarnik_pin" => "Latarnik PIN",
        "latarnik_unlock" => "OPEN LATARNIK",
        "latarnik_use_new_invite" => "I HAVE A NEW INVITATION",
        "latarnik_open_web" => "OPEN IN BROWSER ↗",
        "latarnik_not_street_team" => {
            "Latarnik is not a street team or a task-for-rewards program. Help and coverage are always voluntary."
        }
        "latarnik_name" => "Latarnik",
        "latarnik_lock" => "Lock Latarnik",
        "latarnik_my_signal" => "My Signal",
        "latarnik_tab_briefing" => "Briefing",
        "latarnik_tab_radar" => "Radar",
        "latarnik_tab_press" => "Press Room",
        "latarnik_tab_access" => "Access",
        "latarnik_briefing_label" => "YOUR BRIEFING",
        "latarnik_briefing_title" => "Only relevant signals.",
        "latarnik_briefing_subtitle" => {
            "What matters now: local shows, press assets and anything that needs your attention."
        }
        "latarnik_near_you" => "IN YOUR AREA",
        "latarnik_open_radar" => "OPEN RADAR",
        "latarnik_local_signals" => "local signals",
        "latarnik_open_requests" => "open requests",
        "latarnik_allocations" => "allocations",
        "latarnik_news_label" => "VIRYA · NEWSROOM",
        "latarnik_news_title" => "What is new",
        "latarnik_read" => "READ ↗",
        "latarnik_radar_label" => "LOCAL RADAR",
        "latarnik_radar_title" => "Shows within your radius",
        "latarnik_radar_subtitle" => "Choose only the events that are genuinely relevant to you.",
        "latarnik_interested" => "I AM INTERESTED",
        "latarnik_can_help" => "I CAN HELP",
        "latarnik_not_this_time" => "NOT THIS TIME",
        "latarnik_help_kind" => "How you can help",
        "latarnik_help_article" => "Article / publication",
        "latarnik_help_radio" => "Radio",
        "latarnik_help_podcast" => "Podcast",
        "latarnik_help_photos" => "Photos",
        "latarnik_help_share" => "Share",
        "latarnik_help_contact" => "Contact / introduction",
        "latarnik_help_other" => "Other",
        "latarnik_details_optional" => "Details — optional",
        "latarnik_confirm_help" => "CONFIRM YOU CAN HELP",
        "latarnik_open_press_room" => "OPEN PRESS ROOM",
        "latarnik_press_label" => "PRESS ROOM",
        "latarnik_press_title" => "Ready-to-use materials",
        "latarnik_open_asset" => "OPEN ↗",
        "latarnik_need_something" => "NEED SOMETHING?",
        "latarnik_request_material" => "Ask the band for material",
        "latarnik_request_accreditation" => "Accreditation / guest list",
        "latarnik_request_photos" => "Press photos",
        "latarnik_request_clean" => "Clean version",
        "latarnik_request_interview" => "Interview",
        "latarnik_request_other" => "Other material",
        "latarnik_send_request" => "SEND REQUEST",
        "latarnik_accreditation_note" => {
            "Accreditation depends on organizer approval and available allocation. Coverage is never a condition of entry."
        }
        "latarnik_coverage_label" => "COLLABORATION RESULT",
        "latarnik_coverage_title" => "Add published coverage",
        "latarnik_coverage_hint" => {
            "If you have already published something, you can add the link. This is voluntary and never a condition of accreditation or access."
        }
        "latarnik_coverage_kind" => "Coverage type",
        "latarnik_coverage_video" => "Video",
        "latarnik_coverage_social" => "Post / social",
        "latarnik_coverage_url" => "HTTPS link",
        "latarnik_coverage_title_optional" => "Title — optional",
        "latarnik_coverage_submit" => "ADD COVERAGE",
        "latarnik_coverage_saved" => "Coverage saved. Thank you.",
        "latarnik_access_label" => "ACCESS",
        "latarnik_access_title" => "Accreditation and allocations",
        "latarnik_access_subtitle" => {
            "Request status, guest list and selected physical promo allocations in one place."
        }
        "latarnik_requests" => "Your requests",
        "latarnik_release_allocations" => "Selected releases",
        "latarnik_claim_until" => "Confirm by",
        "latarnik_confirm_delivery" => "CONFIRM DELIVERY",
        "latarnik_decline" => "NOT THIS TIME",
        "latarnik_decline_release_confirm_title" => "RELEASE THIS RESERVED COPY?",
        "latarnik_decline_release_confirm_hint" => {
            "{0} will return to the allocation pool. This action is never completed by a single accidental tap."
        }
        "latarnik_delivery_details" => "Delivery details",
        "latarnik_phone" => "Phone",
        "latarnik_recipient_name" => "Recipient name",
        "latarnik_parcel_locker" => "Parcel locker code",
        "latarnik_save_delivery" => "SAVE DETAILS",
        "latarnik_settings" => "LATARNIK SETTINGS",
        "latarnik_preferences" => "Radius and notifications",
        "latarnik_nearby_push" => "Notify me about relevant nearby shows",
        "latarnik_logout_device" => "Log Latarnik out on this device",
        "latarnik_logout_confirm_hint" => {
            "This revokes the current session and removes Latarnik access from this device. Returning will require a new invitation."
        }
        "latarnik_leave" => "Leave the Latarnik channel",
        "latarnik_leave_confirm_hint" => {
            "This disables the Latarnik channel and all of its sessions without setting a global do-not-contact preference for VIRYA."
        }
        "latarnik_do_not_contact" => "Do not contact me",
        "latarnik_dnc_confirm_hint" => {
            "This sets a global do-not-contact preference: Latarnik is disabled and VIRYA must not use this relationship for outreach."
        }
        "latarnik_confirm_action" => "CONFIRM",
        "affiliate_disclosure" => {
            "Affiliate links. A purchase may earn VIRYA a commission at no extra cost to you."
        }
        "affiliate_eyebrow" => "VIRYA GEAR",
        "affiliate_general_cta" => "START THROUGH VIRYA ↗",
        "affiliate_general_note" => {
            "Start through VIRYA. Your purchase can help support future shows and projects."
        }
        "affiliate_general_title" => "Already shopping at Thomann?",
        "affiliate_intro" => {
            "No sponsor catalogue. Just equipment that is genuinely part of our live rig."
        }
        "affiliate_product_cta" => "VIEW AT THOMANN ↗",
        "affiliate_product_note" => "Our main guitar processor and the centre of the live rig.",
        "affiliate_section_aria" => "VIRYA gear and Thomann affiliate links",
        "affiliate_title" => "Gear we actually use",
        "affiliate_used_live" => "USED LIVE",
        _ => key,
    }
}
