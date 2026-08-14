pub(super) fn text(key: &'static str) -> &'static str {
    match key {
        "back_signal" => "← SYGNAŁ",
        "are_you_on_the_staff" => "JESTEŚ W STAFFIE?",
        "enter_the_staff_password_used_in_the" => "Podaj hasło staff używane w panelu Virya.",
        "zone_prefix" => "Strefa ",
        "team_zone_suffix" => "zespołu.",
        "gate_sales_and_show_operations_access_is" => {
            "Dostęp do bramki, sprzedaży i obsługi koncertu jest oddzielony od konta fana."
        }
        "staff_verification" => "WERYFIKACJA STAFF",
        "virya_panel_password" => "Hasło panelu Virya",
        "use_the_same_password_as_in_qr" => {
            "Użyj tego samego hasła co w QR, bramce i Control Center. Po weryfikacji aplikacja pokaże lokalny PIN lub parowanie urządzenia."
        }
        "staff_password" => "Hasło staff",
        "checking" => "SPRAWDZAM…",
        "open_staff_zone" => "OTWÓRZ STREFĘ STAFF",
        "password_is_verified_by_virya_music_and" => {
            "Hasło jest sprawdzane po stronie virya.music, nie jest zapisywane w aplikacji."
        }
        "failed_to_read_the_staff_vault" => "NIE UDAŁO SIĘ ODCZYTAĆ SEJFU STAFF",
        "checking_the_secure_vault" => "SPRAWDZAM BEZPIECZNY SEJF",
        "virya_staff" => "Virya staff",
        "pin_must_contain_at_least_4_characters" => "PIN musi mieć co najmniej 4 znaki.",
        "enter_a_4_6_digit_pin_and" => "Podaj 4–6-cyfrowy PIN i zeskanuj albo wklej kod parowania.",
        "enter_a_4_6_digit_pin_and_2" => "Podaj 4–6-cyfrowy PIN i poprawny token urządzenia.",
        "code_scanned" => "Kod zeskanowany",
        "enter_the_pin_below_and_tap_pair" => "Wpisz PIN poniżej i kliknij SPARUJ.",
        "connecting" => "ŁĄCZĘ…",
        "scan_qr_code" => "ZESKANUJ KOD QR",
        "code_shown_in_the_virya_panel" => "Kod pokazany w panelu Virya",
        "or_label" => "ALBO",
        "pairing_code" => "Kod parowania",
        "create_an_unlock_pin" => "Utwórz PIN do odblokowania",
        "enter_4_6_digits_for_example_2580" => {
            "Wpisz 4–6 cyfr, np. 2580. To PIN tylko do Virya Signal — nie kod QR ani PIN telefonu."
        }
        "pin_example" => "np. 2580",
        "hide_manual_settings" => "UKRYJ USTAWIENIA RĘCZNE",
        "advanced_settings" => "USTAWIENIA ZAAWANSOWANE",
        "device_person_name" => "Nazwa urządzenia / osoby",
        "device_token" => "Token urządzenia",
        "save_manually" => "ZAPISZ RĘCZNIE",
        "app_unlock_pin" => "PIN do odblokowania aplikacji",
        "enter_the_pin_created_when_this_device" => {
            "Wpisz PIN ustawiony podczas parowania tego urządzenia."
        }
        "your_pin" => "Twój PIN",
        "unlock" => "ODBLOKUJ",
        "open_menu" => "Otwórz menu",
        "close_and_lock_panel" => "Zamknij i zablokuj panel",
        "discounts" => "Zniżki",
        "qr_codes" => "Kody QR",
        "settings" => "Ustawienia",
        "home_tab" => "Start",
        "album_experience" => "Doświadczenie albumu",
        "synesthesia_five_album_draw" => {
            "Przejdź wszystkie 11 pokojów. Ukończenie daje 1 los w osobnej puli 5 płyt."
        }
        "enter_synesthesia" => "WEJDŹ DO SYNESTHESII",
        "synesthesia_best_time" => "Najlepszy czas {}",
        "synesthesia_rank" => "miejsce #{}",
        "synesthesia_rooms_progress" => "{}/11 pokojów",
        "synesthesia_rooms_done" => "{}/11 pokojów · ukończono",
        "synesthesia_runs_count" => "{} przebiegów",
        "your_signal_now" => "TWÓJ SYGNAŁ TERAZ",
        "your_participation" => "TWÓJ ŚLAD",
        "participation_history_title" => "W czym byłeś częścią",
        "participation_history_hint" => "Historia uczestnictwa, bez punktów i leveli.",
        "synesthesia_journey" => "podróż Synesthesia",
        "area_discoveries" => "odkrycia AREA",
        "concert_orders" => "zamówienia biletów",
        "concert_passes" => "wejściówki koncertowe",
        "signal_city_context" => "Najbliższy kontekst: {}.",
        "signal_home_context" => "Jedno miejsce na koncerty, bilety, AREA i postęp albumu.",
        "cached_data" => "DANE Z CACHE",
        "wallet_cached_offline" => "OFFLINE — ZASZYFROWANA KOPIA PORTFELA",
        "journey_completed" => "Podróż ukończona",
        "continue_the_journey" => "Kontynuuj podróż",
        "start_the_journey" => "Rozpocznij podróż",
        "completion_linked_to_signal" => "Ukończenie jest już połączone z Twoim Sygnałem.",
        "completion_saved_link_it_to_signal" => {
            "Ukończenie zapisane. Otwórz Synesthesię, aby połączyć je z profilem."
        }
        "rooms_completed_count" => "Pokoje świadomości ukończone w tej podróży.",
        "open_synesthesia" => "OTWÓRZ SYNESTHESIĘ",
        "next_signal" => "NASTĘPNY SYGNAŁ",
        "show_details" => "SZCZEGÓŁY",
        "active_passes" => "aktywne wejściówki",
        "area_findings" => "znaleziska AREA",
        "signal_home_unavailable" => "Snapshot Sygnału jest chwilowo niedostępny",
        "signal_home_fallback_hint" => {
            "Pozostałe zakładki nadal działają niezależnie. Spróbuj odświeżyć za chwilę."
        }
        "signal_tab" => "Sygnał",
        "scan_tab" => "Skan",
        "tickets_tab" => "Bilety",
        "shows_tab" => "Koncerty",
        "store_tab" => "Sklep",
        "profile_tab" => "Profil",
        "area_game_tab" => "Gra AREA",
        "staff_zone" => "Strefa staff",
        "close_and_lock_signal" => "Zamknij i zablokuj Sygnał",
        "shows_count_label" => "koncerty",
        "active_qr" => "aktywne QR",
        "check_ins" => "check-iny",
        "no_upcoming_shows" => "Brak nadchodzących koncertów",
        "new_events_will_appear_here" => "Kiedy pojawi się nowe wydarzenie, zobaczysz je tutaj.",
        "owner_only_view" => "Widok tylko dla ownera",
        "consent_growth_and_city_statistics_are_available" => {
            "Statystyki zgód, wzrostu i miast są dostępne wyłącznie dla właściciela."
        }
        "data_is_aggregated_in_crowdrelay_and_contains" => {
            "Dane są agregowane w CrowdRelay i nie zawierają adresów e-mail ani identyfikatorów fanów."
        }
        "refreshing" => "ODŚWIEŻAM…",
        "refresh" => "ODŚWIEŻ",
        "no_signal_snapshot" => "Brak snapshotu Sygnału",
        "refresh_the_data_if_the_backend_is" => {
            "Odśwież dane. Jeżeli backend jest jeszcze w trakcie wdrożenia, panel pokaże bezpieczny błąd zamiast pustego ekranu."
        }
        "partial_snapshot_unavailable_sources" => "Snapshot częściowy. Niedostępne źródła: {}.",
        "no_city_aggregate" => "Brak agregatu miast",
        "signal_has_no_confirmed_city_data_yet" => {
            "Sygnał nie ma jeszcze potwierdzonych danych miejskich albo źródło jest chwilowo niedostępne."
        }
        "active" => "aktywni",
        "marketing_consents" => "zgody marketingowe",
        "new_30_days" => "nowi / 30 dni",
        "confirmed_among_active_and_pending" => "potwierdzonych spośród aktywnych i oczekujących",
        "all" => "wszyscy",
        "pending" => "oczekujący",
        "unsubscribed" => "wypisani",
        "muted" => "wyciszeni",
        "nearby_notifications" => "powiadomienia w pobliżu",
        "activity" => "Aktywność",
        "text_30_days_total" => "30 dni / całość",
        "new_7_days" => "nowi / 7 dni",
        "referrals" => "polecenia",
        "show_interests" => "zainteresowania koncertami",
        "nearby_notifications_2" => "powiadomienia nearby",
        "cities_awaiting_moderation" => "miasta do moderacji",
        "strongest_cities" => "Najsilniejsze miasta",
        "snapshot_generated_at_aggregated_data_only" => {
            "Snapshot: {generated_at}. Dane wyłącznie zagregowane."
        }
        "select_a_show_first" => "Najpierw wybierz koncert.",
        "select_a_show" => "Wybierz koncert.",
        "snapshot_ready_durable_tickets" => "Snapshot gotowy: {} trwałych biletów.",
        "sync_saved_conflicts_still_pending" => "Sync: {} zapisane, {} konfliktów, {} nadal czeka.",
        "show_data_removed_from_the_device" => "Dane koncertu usunięte z urządzenia.",
        "show" => "Koncert",
        "loading_shows" => "Ładuję koncerty…",
        "select_an_event" => "Wybierz wydarzenie",
        "gate_works_locally" => "Bramka pracuje lokalnie",
        "works_without_lte" => "Odporność na brak LTE",
        "download_a_secure_snapshot_before_opening_the" => {
            "Pobierz bezpieczny snapshot przed otwarciem bram."
        }
        "prepare_offline" => "PRZYGOTUJ OFFLINE",
        "sync" => "SYNCHRONIZUJ",
        "clear" => "WYCZYŚĆ",
        "verifying" => "WERYFIKUJĘ…",
        "scan_locally" => "SKANUJ LOKALNIE",
        "open_camera" => "URUCHOM APARAT",
        "durable_t1_ticket_qr_only" => "Wyłącznie trwały QR biletu t1",
        "ticket_or_admission_pass_qr" => "QR biletu lub wejściówki",
        "qr_code_or_admission_pass_number" => "Kod QR lub numer wejściówki",
        "check" => "SPRAWDŹ",
        "select_a_show_and_enter_the_fan" => "Wybierz koncert i podaj e-mail fana.",
        "enter_the_admission_pass_public_reference" => "Podaj public reference wejściówki.",
        "admission_pass_has_been_revoked" => "Wejściówka została unieważniona.",
        "select_a_show_2" => "Wybierz koncert",
        "sold" => "sprzedane",
        "in_checkout" => "w trakcie",
        "available_label" => "dostępne",
        "refunds" => "zwroty: {}",
        "admission_pass_number_is_a_safe_public" => {
            "Numer wejściówki to bezpieczny publiczny identyfikator, np. VRY-... Nie jest tokenem QR ani prywatnym tokenem zamówienia."
        }
        "issue_pass" => "WYDAJ WEJŚCIÓWKĘ",
        "admission_pass_number_e_g_vry" => "Numer wejściówki, np. VRY-…",
        "revoke" => "UNIEWAŻNIJ",
        "enter_the_code_and_sale_number" => "Podaj kod i numer sprzedaży.",
        "discount_code" => "Kod zniżkowy",
        "sale_number" => "Numer sprzedaży",
        "redeem_coupon" => "ZREALIZUJ KUPON",
        "usage" => "Użycie {}/{}",
        "main_entrance" => "Wejście główne",
        "enter_a_valid_start_date" => "Podaj poprawny początek ważności.",
        "enter_a_valid_end_date" => "Podaj poprawny koniec ważności.",
        "limit_must_be_a_positive_number" => "Limit musi być dodatnią liczbą.",
        "campaign_end_must_be_after_its_start" => "Koniec kampanii musi być później niż początek.",
        "select_a_show_and_name_the_campaign" => "Wybierz koncert i nazwij kampanię.",
        "qr_campaign_created" => "Kampania QR utworzona.",
        "loading_campaigns" => "Ładuję kampanie…",
        "point_campaign_name" => "Nazwa punktu / kampanii",
        "valid_from" => "Ważna od",
        "valid_until" => "Ważna do",
        "check_in_limit_optional" => "Limit check-inów (opcjonalnie)",
        "create_campaign" => "UTWÓRZ KAMPANIĘ",
        "campaign_has_been_disabled" => "Kampania została wyłączona.",
        "disable_campaign" => "WYŁĄCZ KAMPANIĘ",
        "limit_v" => "limit {v}",
        "no_limit" => "bez limitu",
        "connection" => "Połączenie",
        "permissions" => "Uprawnienia",
        "refresh_all_data" => "Odśwież wszystkie dane",
        "lock_panel" => "Zablokuj panel",
        "remove_operator_profile" => "Usuń profil operatora",
        "operator_token_is_stored_in_an_encrypted" => {
            "Token operatora przechowuje zaszyfrowany sejf Stronghold. Warstwa WebView nigdy go nie odczytuje."
        }
        "refresh_2" => "Odśwież",
        "cockpit_is_partially_available_unavailable" => {
            "Cockpit działa częściowo. Niedostępne: {}."
        }
        "no_error_code" => "brak kodu błędu",
        "attempt" => "próba {}/{}",
        "retry_had_already_been_accepted" => "Retry był już wcześniej przyjęty.",
        "item_returned_to_the_queue" => "Wpis wrócił do kolejki.",
        "language" => "Język",
        "app_language" => "Język aplikacji",
        "changing_the_language_reloads_the_interface_your" => {
            "Zmiana języka przeładuje interfejs. Dane i sesja pozostaną bez zmian."
        }
        "polish" => "Polski",
        "english" => "Angielski",
        "failed_to_read_the_fan_profile" => "NIE UDAŁO SIĘ ODCZYTAĆ PROFILU FANA",
        "checking_your_signal" => "SPRAWDZAM TWÓJ SYGNAŁ",
        "your_profile_remains_untouched" => "Profil pozostaje nietknięty.",
        "app_will_not_continue_to_signup_or" => {
            "Aplikacja nie przejdzie do rejestracji ani parowania, dopóki nie potwierdzi stanu zaszyfrowanego sejfu na urządzeniu."
        }
        "try_again" => "SPRÓBUJ PONOWNIE",
        "enter_the_email_used_to_join_signal" => "Podaj e-mail użyty przy zapisie do Sygnału.",
        "paste_the_code_or_full_link_or" => "Wklej kod, cały link albo zeskanuj QR z wiadomości.",
        "create_a_local_pin_with_at_least" => "Ustaw lokalny PIN mający co najmniej 4 znaki.",
        "marketing_consent_is_required_to_join_signal" => {
            "Zgoda marketingowa jest wymagana do dołączenia do Sygnału."
        }
        "could_not_save_the_city_message" => "Nie udało się zapisać miasta: {message}",
        "select_a_city_or_enter_your_own" => "Wybierz miasto albo wpisz własne.",
        "we_sent_a_secure_access_link_scan" => {
            "Wysłaliśmy bezpieczny link dostępu. Zeskanuj QR albo wklej kod z wiadomości."
        }
        "we_sent_a_confirmation_code_scan_the" => {
            "Wysłaliśmy kod potwierdzający. Zeskanuj QR albo wklej kod z wiadomości."
        }
        "new_message_was_not_sent_because_the" => {
            "Nowa wiadomość nie została wysłana, bo poprzedni kod jest jeszcze ważny. Użyj poprzedniej wiadomości albo spróbuj ponownie za około {minutes} min."
        }
        "request_was_accepted_check_your_inbox_and" => {
            "Zgłoszenie zostało przyjęte. Sprawdź skrzynkę i spam; jeśli wiadomości nie ma, spróbuj ponownie później."
        }
        "enter_the_email_used_in_virya_signal" => "Podaj e-mail użyty w Virya Signal.",
        "if_this_email_is_registered_in_virya" => {
            "Jeśli ten e-mail jest zapisany w Virya Signal, wysłaliśmy świeży link logowania z QR. Po otwarciu ustaw nowy PIN dla tego urządzenia."
        }
        "qr_scanned_enter_your_email_and_local" => {
            "QR zeskanowany. Uzupełnij e-mail i lokalny PIN."
        }
        "shows_tickets" => "Koncerty, bilety",
        "and_rewards" => "i nagrody.",
        "join_in_3_steps" => "Dołącz w 3 krokach:",
        "how_to_join" => "Jak dołączyć",
        "enter_your_email_and_city" => " Podaj e-mail i miasto",
        "confirm_the_code_from_the_message" => " Potwierdź kod z wiadomości",
        "discover_shows_near_you" => " Odkrywaj koncerty blisko Ciebie",
        "what_virya_signal_gives_you" => "Co daje Virya Signal",
        "shows_near_you" => " koncerty blisko Ciebie",
        "tickets_and_qr_codes_on_your_phone" => " bilety i QR w telefonie",
        "rewards_for_simple_actions" => " nagrody za proste akcje",
        "get_started" => "ZACZYNAM",
        "i_have_a_code" => "MAM KOD",
        "email" => "E-mail",
        "name_optional" => "Imię / nazwa (opcjonalnie)",
        "fastest_scan_the_qr_from_the_email" => "Najszybciej: zeskanuj QR z maila.",
        "you_can_also_paste_the_full_link" => {
            "Możesz też wkleić cały link albo 64-znakowy kod. Aplikacja sama wyciągnie właściwy token."
        }
        "email_link_or_code" => "Link lub kod z e-maila",
        "paste_a_link_or_code_or_use" => "Wklej link, kod albo użyj QR",
        "scan_qr" => "SKANUJ QR",
        "or_hold_the_field_above_and_choose" => "albo przytrzymaj pole wyżej i wybierz Wklej",
        "local_pin" => "Lokalny PIN",
        "pin_encrypts_your_profile_on_this_device" => {
            "PIN szyfruje profil tylko na tym urządzeniu. Nie wysyłamy go do CrowdRelay."
        }
        "confirm_and_enter" => "POTWIERDŹ I WEJDŹ",
        "i_already_have_an_account_send_login" => "MAM JUŻ KONTO — WYŚLIJ LINK LOGOWANIA",
        "no_message_check_spam_after_15_minutes" => {
            "Nie ma wiadomości? Sprawdź spam. Po 15 minutach wróć do ZACZYNAM i wyślij kod ponownie."
        }
        "city" => "Miejscowość",
        "e_g_bielawa" => "np. Bielawa",
        "province_region_optional" => "Województwo / region (opcjonalnie)",
        "lower_silesia" => "dolnośląskie",
        "enter_your_city_manually_we_will_match" => {
            "Wpisz miejscowość ręcznie — dopasujemy ją do mapy Sygnału."
        }
        "notify_me_about_nearby_shows" => "Powiadamiaj mnie o koncertach w pobliżu",
        "referral_code_optional" => "Kod polecający (opcjonalnie)",
        "i_want_to_receive_information_about_virya" => {
            "Chcę otrzymywać informacje o koncertach, premierach i nagrodach Viryi."
        }
        "join_signal" => "DOŁĄCZ DO SYGNAŁU",
        "open_my_signal" => "OTWÓRZ MÓJ SYGNAŁ",
        "i_forgot_my_pin_sign_in_again" => "NIE PAMIĘTAM PIN-U / ZALOGUJ PONOWNIE",
        "access_recovery" => "ODZYSKIWANIE DOSTĘPU",
        "create_a_new_pin" => "Ustaw nowy PIN",
        "enter_your_email_request_a_fresh_link" => {
            "Podaj e-mail, wyślij świeży link, a potem zeskanuj QR lub wklej kod w pole."
        }
        "send_login_link" => "WYŚLIJ LINK LOGOWANIA",
        "paste_link_or_code" => "Wklej link lub kod",
        "or_hold_the_field_and_choose_paste" => "albo przytrzymaj pole i wybierz Wklej",
        "new_local_pin" => "Nowy lokalny PIN",
        "confirm_and_set_new_pin" => "POTWIERDŹ I USTAW NOWY PIN",
        "back_to_pin_login" => "WRÓĆ DO LOGOWANIA PIN-EM",
        "code" => "Kod: {}",
        "loading_signal" => "Ładowanie Sygnału…",
        "entries" => "losy",
        "coupons" => "kupony",
        "draw" => "Losowanie {}",
        "proof" => "DOWÓD ↗",
        "merch" => "Merch",
        "products_and_bundles_use_the_same_inventory" => {
            "Produkty i zestawy korzystają z tego samego stanu magazynowego co sklep online. Płatność otworzy bezpieczny Stripe Checkout, a aplikacja nie przechowuje danych karty."
        }
        "store_is_temporarily_unavailable" => "Sklep jest chwilowo niedostępny",
        "rest_of_signal_is_working_normally_try" => {
            "Pozostałe części Sygnału działają normalnie. Spróbuj ponownie za moment."
        }
        "refresh_merch" => "ODŚWIEŻ MERCH",
        "open_full_store" => "OTWÓRZ PEŁNY SKLEP ↗",
        "bundles" => "ZESTAWY",
        "bundles_from_the_online_store" => "Bundle ze sklepu online",
        "up_to_30" => "DO −30%",
        "bundles_are_currently_unavailable_in_live_inventory" => {
            "Zestawy są teraz niedostępne w live inventory."
        }
        "view_bundles" => "ZOBACZ ZESTAWY ↗",
        "low_stock" => "OSTATNIE SZTUKI",
        "available_status" => "DOSTĘPNY",
        "out_of_stock" => "BRAK NA STANIE",
        "check_again" => "SPRAWDŹ PONOWNIE",
        "buy_in_store" => "KUP W SKLEPIE ↗",
        "bundles_load_independently_from_products" => {
            "Zestawy doczytują się niezależnie od produktów."
        }
        "individual_products" => "POJEDYNCZE PRODUKTY",
        "choose_your_merch" => "Wybierz swój merch",
        "pre_order" => "PRZEDSPRZEDAŻ",
        "could_not_load_store_status" => "Nie udało się pobrać stanu sklepu",
        "shows_tickets_and_profile_remain_available" => {
            "Koncerty, bilety i profil pozostają dostępne."
        }
        "no_shows_in_the_calendar" => "Brak koncertów w kalendarzu",
        "new_events_will_appear_here_2" => "Kiedy pojawi się nowy event, będzie tutaj.",
        "show_saved_to_your_signal" => "Koncert zapisany w Twoim Sygnale.",
        "saving" => "ZAPISUJĘ…",
        "saved" => "✓ MAM TO",
        "interested" => "+ INTERESUJE MNIE",
        "claimed" => "ODEBRANE",
        "redeemed" => "WYKORZYSTANE",
        "buy_ticket" => "KUP BILET",
        "back_back_to_shows" => "← WRÓĆ DO KONCERTÓW",
        "could_not_check_ticket_sales" => "Nie udało się sprawdzić sprzedaży",
        "no_virya_ticket_pool" => "Brak własnej puli Virya",
        "you_can_open_the_show_page_or" => {
            "Możesz przejść do strony koncertu lub sprzedaży prowadzonej przez organizatora."
        }
        "check_tickets" => "SPRAWDŹ BILETY ↗",
        "ticket_sales_will_open_soon" => "Sprzedaż rozpocznie się wkrótce.",
        "online_sales_have_ended" => "Sprzedaż online została zakończona.",
        "this_ticket_pool_is_sold_out" => "Ta pula biletów jest wyprzedana.",
        "ticket_sales_are_temporarily_disabled" => "Sprzedaż jest chwilowo wyłączona.",
        "this_show_is_not_currently_on_sale" => {
            "Ten koncert nie jest obecnie dostępny w sprzedaży."
        }
        "tickets_are_not_available_right_now" => "Bilety nie są teraz dostępne.",
        "select_tickets_places_will_be_reserved_while" => {
            "Wybierz bilety. Miejsca zostaną zarezerwowane na czas płatności."
        }
        "in_checkout_2" => "w płatności",
        "open_the_show_page" => "Sprawdź stronę koncertu",
        "if_the_organiser_runs_a_separate_ticket" => {
            "Jeżeli organizator prowadzi osobną sprzedaż, znajdziesz ją pod tym przyciskiem."
        }
        "select_at_least_one_ticket" => "Wybierz co najmniej jeden bilet.",
        "order_saved_complete_the_secure_stripe_payment" => {
            "Zamówienie {} zapisane. Dokończ bezpieczną płatność Stripe."
        }
        "payment_opened_for_order" => "Otworzono płatność dla zamówienia {}.",
        "available" => "Dostępne: {}",
        "ticket_quantity" => "Liczba biletów",
        "decrease_ticket_quantity" => "Zmniejsz liczbę biletów",
        "increase_ticket_quantity" => "Zwiększ liczbę biletów",
        "name_on_the_order_optional" => "Imię i nazwisko na zamówieniu (opcjonalnie)",
        "tickets_and_confirmation_will_be_sent_to" => "Bilety i potwierdzenie trafią na {}",
        "tickets_will_be_sent_to_the_fan" => "Bilety trafią na e-mail konta fana.",
        "invoice_full_form" => "FAKTURA / PEŁNY FORMULARZ ↗",
        "selected_tickets" => "Wybrane bilety",
        "gross_total" => "Razem brutto",
        "reserving" => "REZERWUJĘ…",
        "order_saved" => "ZAMÓWIENIE ZAPISANE",
        "continue_to_stripe_payment" => "PRZEJDŹ DO PŁATNOŚCI STRIPE",
        "reopen_payment" => "OTWÓRZ PŁATNOŚĆ PONOWNIE ↗",
        "card_details_never_reach_virya_signal_payment" => {
            "Dane karty nie trafiają do Virya Signal. Płatność otworzy się w bezpiecznym Stripe Checkout."
        }
        "open_map_and_start" => "OTWÓRZ MAPĘ I ZACZNIJ",
        "refresh_progress" => "Odśwież progres",
        "open_area" => "OTWÓRZ AREA",
        "enter_the_order_id_and_private_token" => {
            "Podaj identyfikator zamówienia i prywatny token."
        }
        "tickets_saved_to_the_wallet" => "Bilety zapisane w portfelu.",
        "paste_the_admission_pass_token" => "Wklej token wejściówki.",
        "admission_pass_assigned_to_this_device" => "Wejściówka przypisana do urządzenia.",
        "show_entry_qr" => "POKAŻ QR NA WEJŚCIE",
        "token_from_the_message" => "Token z wiadomości",
        "claim_admission_pass" => "ODBIERZ WEJŚCIÓWKĘ",
        "add_an_existing_order" => "Dodaj istniejące zamówienie",
        "order_uuid" => "UUID zamówienia",
        "private_checkout_token" => "Prywatny checkout token",
        "add_to_wallet" => "DODAJ DO PORTFELA",
        "we_resent_the_wallet_by_email" => "Wysłaliśmy ponownie portfel na e-mail.",
        "sending" => "Wysyłam…",
        "resend_tickets_by_email" => "Wyślij bilety ponownie na e-mail",
        "generating" => "GENERUJĘ…",
        "hide_qr" => "UKRYJ QR",
        "show_qr" => "POKAŻ QR",
        "qr_unavailable" => "QR NIEDOSTĘPNY",
        "qr_valid_until" => "QR ważny do {}",
        "valid_until_2" => "ważny do {}",
        "my_profile" => "MÓJ PROFIL",
        "signal_settings" => "Ustawienia Sygnału",
        "virya_fan" => "Fan Viryi",
        "orders" => "zamówienia",
        "admission_passes" => "wejściówki",
        "refreshing_2" => "Odświeżam…",
        "refresh_data" => "Odśwież dane",
        "lock_app" => "Zablokuj aplikację",
        "remove_profile_and_tickets_from_device" => "Usuń profil i bilety z urządzenia",
        "fan_session_admission_pass_and_private_wallet" => {
            "Sesja fana, wejściówka oraz prywatne tokeny portfela są przechowywane w osobnym, zaszyfrowanym sejfie Stronghold."
        }
        "feedback_must_contain_between_8_and_2000" => "Feedback powinien mieć od 8 do 2000 znaków.",
        "feedback_was_sent_anonymously_thank_you" => {
            "Feedback został przyjęty anonimowo. Jeśli jesteś offline, wyślę go automatycznie po powrocie sieci."
        }
        "anonymous_feedback" => "ANONIMOWY FEEDBACK",
        "tell_us_what_to_improve" => "Powiedz nam, co poprawić",
        "app_sends_only_the_category_and_message" => {
            "Aplikacja wysyła tylko kategorię i treść — bez e-maila, nazwy, tokenu sesji i identyfikatora profilu. Hosting może zachować standardowe logi techniczne połączenia."
        }
        "category" => "Kategoria",
        "idea" => "Pomysł",
        "bug_label" => "Błąd",
        "shows_and_tickets" => "Koncerty i bilety",
        "other" => "Inne",
        "message" => "Treść",
        "tell_us_directly_what_is_broken_or" => "Napisz wprost, co działa źle albo czego brakuje…",
        "sending_2" => "WYSYŁAM…",
        "send_anonymously" => "WYŚLIJ ANONIMOWO",
        "loading" => "Ładowanie",
        "feedback_was_sent" => "feedback został wysłany",
        "could_not_refresh_orders_the_remaining_tickets" => {
            "Nie udało się odświeżyć {} zamówień. Pozostałe bilety są dostępne."
        }
        "details_coming_soon" => "Szczegóły wkrótce",
        "venue_coming_soon" => "miejsce wkrótce",
        "scanner_returned_no_code" => "Skaner nie zwrócił kodu.",
        "server_response_has_an_unexpected_format" => "Odpowiedź serwera ma nieoczekiwany format.",
        "response_decoding_error_raw" => "Błąd odczytu odpowiedzi: {raw}",
        "unknown_application_error" => "Nieznany błąd aplikacji",
        "pair" => "Sparuj",
        "device" => "urządzenie.",
        "no_retyping_the_api_role_or_long" => "Bez przepisywania API, roli i długiego sekretu.",
        "pair_2" => "SPARUJ",
        "operator_profile_is_encrypted_locally" => "Profil operatora jest zaszyfrowany lokalnie.",
        "today_under_control" => "Dzisiaj pod kontrolą",
        "next_show" => "NASTĘPNY KONCERT",
        "upcoming" => "Nadchodzące",
        "community_and_growth" => "Społeczność i wzrost",
        "combined_signal_overview_without_fans_personal_data" => {
            "Zbiorczy obraz Sygnału bez danych osobowych fanów."
        }
        "database_health" => "ZDROWIE BAZY",
        "scan_entry" => "Skanuj wejście",
        "tickets_and_admission_passes" => "Bilety i wejściówki",
        "gross_revenue" => "OBRÓT BRUTTO",
        "recent_orders" => "Ostatnie zamówienia",
        "manual_admission_pass" => "Ręczna wejściówka",
        "redeem_a_discount" => "Realizuj zniżkę",
        "fan_coupon_controlled_use" => "kupon fanowski / kontrolowane użycie",
        "coupon_redeemed" => "KUPON ZREALIZOWANY",
        "qr_campaigns" => "Kampanie QR",
        "active_and_historical" => "Aktywne i historyczne",
        "queues_and_deliveries" => "Kolejki i dostawy",
        "dead_deliveries" => "Martwe dostawy",
        "dead_outbox" => "Martwy outbox",
        "no_dead_entries_the_delivery_pipeline_is" => {
            "Brak martwych wpisów. Tor dostaw jest czysty."
        }
        "your_profile_and_tickets_are_encrypted_on" => {
            "Twój profil i bilety są zaszyfrowane na urządzeniu."
        }
        "your_impact" => "TWÓJ WPŁYW",
        "confirmed_referrals" => "potwierdzonych poleceń",
        "your_coupons" => "Twoje kupony",
        "rewards" => "Nagrody",
        "active_draws" => "Aktywne losowania",
        "entries_2" => "LOSÓW",
        "where_we_play" => "GDZIE GRAMY",
        "find_a_point_in_your_city" => "Znajdź punkt w swoim mieście",
        "open_the_map_choose_an_active_point" => {
            "Otwórz mapę, wybierz aktywny punkt i przejdź do niego. Nie musisz zbierać wszystkiego ani jeździć po kraju."
        }
        "connect_your_browser_wallet_to_your_area" => {
            "Połącz portfel przeglądarkowy z kontem na stronie AREA, aby zachować cały postęp."
        }
        "collection_progress" => "POSTĘP KOLEKCJI",
        "discovered_artifacts" => "Odkryte artefakty",
        "map_shows_active_points_and_gets_you" => {
            "Mapa pokazuje aktywne punkty i prowadzi do startu. Dokładna lokalizacja odsłania się w grze dopiero wtedy, gdy jest potrzebna."
        }
        "area_is_temporarily_unavailable" => "AREA chwilowo niedostępna",
        "refresh_the_data_or_open_the_full" => "Odśwież dane albo otwórz pełną grę.",
        "tickets_and_entry" => "Bilety i wejście",
        "virya_admission_pass" => "WEJŚCIÓWKA VIRYA",
        "did_you_win_an_admission_pass" => "WYGRAŁEŚ WEJŚCIÓWKĘ?",
        "assign_it_to_your_phone" => "Przypisz ją do telefonu",
        "ticket_wallet" => "Portfel biletów",
        "individual_products_2" => "Pojedyncze produkty",
        "bundles_2" => "Zestawy",
        "retry" => "RETRY",
        "connecting_2" => "ŁĄCZĘ",
        "online" => "ONLINE",
        "offline_on" => "OFFLINE ON",
        "offline_off" => "OFFLINE OFF",
        "active_status" => "ACTIVE",
        "closed" => "CLOSED",
        "owner" => "OWNER",
        "staff" => "STAFF",
        "device_label" => "DEVICE",
        "mobile_wallet" => "MOBILE WALLET",
        "virya_control" => "VIRYA CONTROL",
        "virya_signal" => "VIRYA SIGNAL",
        "virya_store" => "VIRYA STORE",
        "virya_area" => "VIRYA AREA",
        "virya_tickets" => "VIRYA // BILETY",
        "tickets_pending_conflicts" => "{} biletów · {} oczekuje · {} konfliktów",
        "offline_show_mode_status" => "Status trybu koncertowego offline",
        "eligible_tickets" => "dostępne",
        "pending_scans" => "oczekujące",
        "synced_scans" => "zsynchronizowane",
        "scan_conflicts" => "konflikty",
        "check_ins_2" => "{} check-inów",
        "attempt_2" => "{} · próba {}/{}",
        "my_signal" => "Mój Sygnał",
        "pending_2" => "oczekujące",
        "message_order_is_saved_use_the_reopen" => {
            "{message} Zamówienie {} jest zapisane — użyj przycisku ponownego otwarcia płatności."
        }
        "reward_credits_credits" => "{reward_credits} kredytów",
        "live_count_active_points" => "{live_count} aktywne punkty",
        "voucher_count_rewards" => "{voucher_count} nagrody",
        "community_percent_community" => "{community_percent}% społeczności",
        "sent" => "wysłaliśmy",
        "revoked" => "unieważnion",
        "jan" => "STY",
        "feb" => "LUT",
        "mar" => "MAR",
        "apr" => "KWI",
        "may" => "MAJ",
        "jun" => "CZE",
        "jul" => "LIP",
        "aug" => "SIE",
        "sep" => "WRZ",
        "oct" => "PAŹ",
        "nov" => "LIS",
        "dec" => "GRU",
        "text" => "---",
        "active_2" => "{} aktywnych",
        "virya_merch_bundle" => "{} — zestaw merchu Virya",
        "virya_merch" => "{} — merch Virya",
        "virya_show" => "{} — koncert Virya",
        "native_app_bridge_is_unavailable" => "Natywny most aplikacji nie jest dostępny.",
        "operation_command_timed_out" => "Operacja {command} przekroczyła limit czasu.",
        "camera_permission_module_is_unavailable_in_this" => {
            "Moduł uprawnień aparatu nie jest dostępny w tej wersji aplikacji."
        }
        "camera_access_is_denied_enable_camera_for" => {
            "Brak dostępu do aparatu. Włącz Aparat dla Virya Signal w ustawieniach aplikacji."
        }
        "qr_code_scanner" => "Skaner kodu QR",
        "scan_qr_code_2" => "SKANUJ KOD QR",
        "place_the_code_inside_the_frame" => "Umieść kod wewnątrz ramki",
        "back_cancel_scanning" => "← ANULUJ SKANOWANIE",
        "closing" => "ZAMYKAM…",
        "scanner_is_available_only_in_the_ios" => {
            "Skaner jest dostępny tylko w aplikacji iOS/Android."
        }
        "type" => "Rodzaj",
        "time" => "Czas",
        "operation" => "Operacja",
        "path" => "Ścieżka",
        "previous_launch_ended_with_an_error" => "Poprzednie uruchomienie zakończyło się błędem",
        "app_caught_an_error" => "Aplikacja zatrzymała błąd",
        "we_do_not_hide_failures_copy_the" => {
            "Nie ukrywamy awarii. Skopiuj raport i wyślij go razem z informacją, co było kliknięte."
        }
        "copy_report" => "KOPIUJ RAPORT",
        "restart_app" => "URUCHOM PONOWNIE",
        "close" => "ZAMKNIJ",
        "report_copied" => "Raport skopiowany.",
        "press_and_hold_the_report_text_and" => "Przytrzymaj tekst raportu i skopiuj ręcznie.",
        "previous_launch_interrupted_operation_command" => {
            "Poprzednie uruchomienie przerwało operację {command}."
        }
        "previous_launch_ended_without_a_clean_shutdown" => {
            "Poprzednie uruchomienie zakończyło się bez czystego zamknięcia."
        }
        "virya_signal_diagnostics" => "VIRYA SIGNAL / DIAGNOSTYKA",
        "native_error_not_configured" => "Profil urządzenia nie jest skonfigurowany",
        "native_error_invalid_pin" => "Nieprawidłowy PIN",
        "native_error_locked" => "Sesja jest zablokowana",
        "native_error_unauthorized" => {
            "Token urządzenia jest nieprawidłowy albo nie ma wymaganych uprawnień"
        }
        "native_error_forbidden" => "Ta operacja wymaga roli owner",
        "native_error_conflict" => "Konflikt",
        "native_error_not_found" => "Nie znaleziono danych",
        "native_error_crowdrelay" => "CrowdRelay",
        "native_error_network" => "Błąd sieci",
        "native_error_url" => "Błędny URL",
        "native_error_data" => "Błąd danych",
        "native_error_file" => "Błąd pliku",
        "native_error_vault" => "Błąd magazynu sejfu",
        "native_error_background_task" => "Wewnętrzny błąd zadania",
        "native_pin_4_128" => "PIN musi mieć 4–128 znaków",
        "native_damaged_device_profile" => "Uszkodzony profil urządzenia",
        "native_invalid_device_name" => "Nieprawidłowa nazwa urządzenia",
        "native_invalid_device_token" => "Nieprawidłowy token urządzenia",
        "native_complete_fan_data" => "Uzupełnij poprawnie dane fana",
        "native_paste_valid_code" => "Wklej prawidłowy kod, link albo zeskanuj QR",
        "native_invalid_email_or_token" => "Nieprawidłowy e-mail lub token",
        "native_invalid_pass_data" => "Nieprawidłowe dane wejściówki",
        "native_invalid_qr_campaign_data" => "Nieprawidłowe dane kampanii QR",
        "native_operator_pin_4_6" => "PIN operatora musi mieć 4–6 cyfr",
        "native_pin_min_4" => "PIN musi mieć co najmniej 4 znaki",
        "native_api_must_use_https" => "API musi używać HTTPS",
        "native_invalid_api_base_url" => "Nieprawidłowy bazowy URL API",
        "native_backend_update_required" => {
            "Serwer wymaga aktualizacji, zanim ta funkcja będzie dostępna."
        }
        "native_invalid_label" => "Nieprawidłowy {label}",
        "native_public_cache_too_large" => "Lokalny cache danych publicznych jest zbyt duży",
        "native_missing_events_cache" => "Backend potwierdził nieistniejący cache koncertów",
        "native_missing_cities_cache" => "Backend potwierdził nieistniejący cache miast",
        "native_missing_merch_cache" => "Backend potwierdził nieistniejący cache merchu",
        "native_invalid_staff_password" => "Nieprawidłowe hasło staff.",
        "native_staff_rate_limited" => {
            "Za dużo prób logowania. Spróbuj ponownie za kilkanaście minut."
        }
        "native_staff_verification_unavailable" => "Weryfikacja staff jest chwilowo niedostępna",
        "native_staff_verification_failed" => "Nie udało się zweryfikować dostępu staff",
        "native_invalid_store_url" => "Nieprawidłowy adres sklepu",
        "native_bundle_catalog_too_large" => "Katalog zestawów jest zbyt duży",
        "native_invalid_merch_bundle" => "Nieprawidłowy zestaw merchu",
        "native_bundle_too_many_items" => "Zestaw merchu ma zbyt wiele pozycji",
        "native_invalid_bundle_offer" => "Nieprawidłowa oferta zestawu",
        "native_invalid_bundle_variant" => "Nieprawidłowy wariant zestawu",
        "native_choose_feedback_category" => "Wybierz kategorię feedbacku",
        "native_feedback_content_label" => "treść feedbacku",
        "native_feedback_failed" => "Nie udało się przekazać feedbacku",
        "native_response_too_large" => "Odpowiedź CrowdRelay jest zbyt duża",
        "native_operation_rejected" => "CrowdRelay odrzucił operację",
        "native_production_api_https" => "Produkcyjny API URL musi używać HTTPS",
        "native_invalid_identifier" => "Nieprawidłowy identyfikator",
        "native_invalid_order_id" => "Nieprawidłowy identyfikator zamówienia",
        "native_invalid_qr_code" => "Nieprawidłowy kod QR",
        "native_ticket_offer_invalid" => "Serwer zwrócił nieprawidłową ofertę biletową",
        "native_ticket_pool_invalid" => "Serwer zwrócił nieprawidłową pulę biletów",
        "native_event_id_invalid" => "Nieprawidłowy identyfikator koncertu",
        "native_buyer_name_too_long" => "Imię i nazwisko jest zbyt długie",
        "native_choose_tickets" => "Wybierz bilety",
        "native_ticket_selection_invalid" => "Nieprawidłowy wybór biletów",
        "native_too_many_tickets" => "Wybrano zbyt wiele biletów",
        "native_payment_url_invalid" => "Serwer zwrócił nieprawidłowy adres płatności",
        "native_order_invalid" => "Serwer zwrócił nieprawidłowe zamówienie",
        "native_order_incomplete" => "Serwer zwrócił niepełne dane zamówienia",
        "native_admission_token_label" => "token wejściówki",
        "native_order_token_label" => "token zamówienia",
        "native_qr_token_label" => "token QR",
        "native_coupon_code_label" => "kod kuponu",
        "native_sale_number_label" => "numer sprzedaży",
        "native_admission_session_missing" => "Backend nie zwrócił sesji wejściówki",
        "native_claim_pass_first" => "Najpierw odbierz wejściówkę",
        "native_code_already_used" => {
            "Ten kod został już wykorzystany. Wróć do ZACZYNAM i poproś o nową wiadomość."
        }
        "native_code_invalid_or_expired" => {
            "Kod jest nieprawidłowy albo wygasł. Poproś o nową wiadomość."
        }
        "native_fan_session_missing" => "Backend potwierdził kod, ale nie zwrócił sesji fana",
        "native_area_wallet_id_invalid" => "Nieprawidłowy identyfikator portfela AREA",
        "native_queue_type_invalid" => "Nieprawidłowy typ kolejki",
        "native_snapshot_time_invalid" => "Snapshot ma nieprawidłowy czas",
        "native_offline_t1_only" => "Tryb offline obsługuje wyłącznie trwałe bilety t1",
        "native_ticket_qr_invalid" => "Nieprawidłowy bilet QR",
        "native_event_invalid" => "Nieprawidłowy koncert",
        "native_snapshot_event_mismatch" => "CrowdRelay zwrócił niezgodny snapshot koncertu",
        "native_snapshot_expired" => "Snapshot koncertu jest nieważny albo wygasł",
        "native_snapshot_too_large" => "Snapshot przekracza bezpieczny limit 10 000 wejść",
        "native_snapshot_integrity_failed" => {
            "Snapshot koncertu nie przeszedł kontroli integralności"
        }
        "native_qr_too_long" => "Kod QR jest zbyt długi",
        "native_snapshot_refresh_required" => {
            "Snapshot koncertu wygasł. Połącz się z siecią i pobierz nowy"
        }
        "native_ticket_not_in_snapshot" => "Bilet nie występuje w podpisanym snapshotcie",
        "native_scan_queue_full" => "Lokalna kolejka skanów jest pełna",
        "native_no_prepared_event" => "Brak przygotowanego koncertu",
        "native_link_too_long" => "Link jest zbyt długi",
        "native_https_links_only" => "Można otwierać wyłącznie bezpieczne linki HTTPS",
        "native_open_link_failed" => "Nie udało się otworzyć linku: {error}",
        "native_city_name_invalid" => "Nieprawidłowa nazwa miasta",
        "native_enter_valid_staff_password" => "Podaj poprawne hasło staff.",
        "native_pairing_code_expired" => "Kod parowania wygasł albo jest nieprawidłowy",
        "native_pairing_code_invalid" => "Nieprawidłowy kod parowania",
        "native_pairing_code_empty" => "Kod parowania nie zawiera danych",
        "native_enter_valid_email" => "Podaj poprawny e-mail",
        "native_wallet_limit" => "Portfel może zawierać maksymalnie {max} zamówienia",
        "native_wrong_order_wallet" => "Backend zwrócił portfel innego zamówienia",
        "native_ticket_reference_invalid" => "Nieprawidłowa referencja biletu",
        "native_ticket_not_on_device" => "Nie znaleziono biletu na urządzeniu",
        "native_qr_token_missing" => "Brak tokenu QR w odpowiedzi backendu",
        "native_qr_token_invalid" => "Nieprawidłowy token QR",
        "native_qr_generation_failed" => "Nie udało się wygenerować kodu QR",
        "boot_starting" => "URUCHAMIAM VIRYA SIGNAL",
        "boot_loading_secure_profile" => "Wczytuję bezpieczny profil urządzenia…",
        "boot_taking_longer" => "To trwa dłużej niż zwykle",
        "boot_still_starting" => {
            "Aplikacja nadal się uruchamia. Możesz poczekać albo spróbować ponownie."
        }
        "boot_retry" => "SPRÓBUJ PONOWNIE",
        "boot_diagnostics" => "DIAGNOSTYKA",
        "boot_launch_failed" => "Nie udało się uruchomić aplikacji",
        "boot_reload_help" => {
            "Uruchom aplikację ponownie. Jeśli problem wróci, otwórz diagnostykę i wyślij raport."
        }
        "boot_previous_terminated" => "Poprzednie uruchomienie zniknęło podczas etapu {phase}.",
        "boot_phase_wasm_loading" => "ŁADUJĘ SILNIK APLIKACJI",
        "boot_phase_wasm_entered" => "URUCHAMIAM INTERFEJS",
        "boot_phase_wasm_initialized" => "KOŃCZĘ START",
        "boot_unknown_error" => "Nieznany błąd uruchamiania",
        "boot_start_stopped" => "START APLIKACJI ZATRZYMANY",
        "boot_module_not_started" => "MODUŁ APLIKACJI NIE ZOSTAŁ URUCHOMIONY",
        "boot_engine_load_failed" => "NIE UDAŁO SIĘ ZAŁADOWAĆ SILNIKA APLIKACJI",
        "boot_engine_no_interface" => "SILNIK NIE URUCHOMIŁ INTERFEJSU",
        "boot_interface_incomplete" => "INTERFEJS NIE ZAKOŃCZYŁ STARTU",
        "boot_start_incomplete" => "START NIE ZAKOŃCZYŁ SIĘ",
        "boot_stage_retry_detail" => {
            "Etap: {phase}. Ponowienie wykona jeden czysty restart WebView."
        }
        "boot_retry_failed" => "PONOWNY START NIE POMÓGŁ",
        "boot_retry_blocked_detail" => {
            "Etap: {phase}. Zapisz ten komunikat; aplikacja nie będzie już wpadać w pętlę restartów."
        }
        "boot_almost_ready" => "JESZCZE CHWILA — KOŃCZĘ START",
        "boot_initial_status" => "URUCHAMIAMY SYGNAŁ",
        "boot_retry_button" => "PONÓW START",
        "boot_noscript" => "Virya Signal wymaga JavaScript/WASM.",
        "network_offline_cached" => "OFFLINE — DANE Z PAMIĘCI NADAL DZIAŁAJĄ",
        "network_restored" => "POŁĄCZENIE WRÓCIŁO",
        "native_bundle_name_label" => "nazwa zestawu",
        "native_bundle_description_label" => "opis zestawu",
        "native_bundle_item_label" => "element zestawu",
        "native_image_url_label" => "adres grafiki",
        "native_store_url_label" => "adres sklepu",
        "native_bundle_variant_label" => "wariant zestawu",
        "native_prepare_offline_event_first" => "Najpierw przygotuj koncert offline",
        "location_module_is_unavailable_in_this_app" => {
            "Moduł lokalizacji nie jest dostępny w tej wersji aplikacji."
        }
        "location_access_is_denied_enable_location_for" => {
            "Brak dostępu do lokalizacji. Włącz lokalizację dla Virya Signal w ustawieniach aplikacji."
        }
        "could_not_read_a_fresh_location_move" => {
            "Niestety to nie jest poprawna lokalizacja. Próbuj dalej!"
        }
        "native_area_claim_invalid" => "Dane potwierdzenia punktu AREA są nieprawidłowe.",
        "native_area_drop_inactive" => "Ten punkt AREA nie jest teraz aktywny.",
        "native_area_challenge_invalid" => {
            "Próba lokalizacji wygasła. Uruchom weryfikację ponownie."
        }
        "native_area_rate_limited" => "Za dużo prób. Odczekaj kilka minut i spróbuj ponownie.",
        "native_area_not_enough_samples" => {
            "Nie udało się zebrać wystarczającej liczby świeżych pomiarów. Pozostań chwilę w miejscu i spróbuj ponownie."
        }
        "native_area_low_accuracy" => {
            "Lokalizacja jest zbyt mało dokładna. Wyjdź na otwartą przestrzeń i spróbuj ponownie."
        }
        "native_area_outside_zone" => "Jesteś poza aktywną strefą punktu AREA.",
        "native_area_drop_full" => "Limit odbiorów tego punktu został już osiągnięty.",
        "native_area_claim_conflict" => "Ta próba została już przetworzona. Odśwież postęp AREA.",
        "native_area_temporary" => {
            "Weryfikacja AREA jest chwilowo niedostępna. Spróbuj ponownie za moment."
        }
        "area_in_the_app" => "AREA W APLIKACJI",
        "choose_an_active_point_and_follow_the" => {
            "Wybierz aktywny punkt, sprawdź kierunek i potwierdź wygraną bez wychodzenia z aplikacji."
        }
        "active_area_point" => "Aktywny punkt AREA",
        "inactive_area_point" => "Punkt oczekuje na aktywację",
        "claimed_area_point" => "Punkt już odkryty",
        "locate_nearest_point" => "ZNAJDŹ NAJBLIŻSZY PUNKT",
        "locating_you" => "LOKALIZUJĘ…",
        "nearest_active_point_is_city" => "Najbliższy aktywny punkt: {city}.",
        "you_are_about_distance_from_city" => "Jesteś około {distance} od punktu w mieście {city}.",
        "no_active_area_points_now" => {
            "Teraz nie ma aktywnych punktów AREA. Mapa nadal pokazuje miasta kolejnych aktywacji."
        }
        "open_route_start" => "OTWÓRZ START TRASY",
        "verify_location_and_win" => "POTWIERDŹ LOKALIZACJĘ I WYGRAJ",
        "verifying_location" => "SPRAWDZAM LOKALIZACJĘ…",
        "area_location_privacy" => {
            "Lokalizacja jest używana tylko podczas tej próby. Aplikacja wysyła kilka świeżych próbek do weryfikacji i nie zapisuje trasy."
        }
        "area_point_won" => "PUNKT ODKRYTY",
        "area_point_already_won" => "TEN PUNKT JUŻ MASZ",
        "area_reward_added" => "Do kolekcji dodano „{track}” i {credits} kredyt nagrody.",
        "area_reward_already_present" => "„{track}” jest już w Twojej kolekcji.",
        "select_an_active_point_first" => "Najpierw wybierz aktywny punkt AREA.",
        "location_accuracy_value" => "Dokładność: ±{accuracy} m",
        "approximate_distance_meters" => "{distance} m",
        "approximate_distance_kilometers" => "{distance} km",
        "open_full_area_game" => "OTWÓRZ PEŁNĄ GRĘ AREA",
        "fan_app_unlock_pin" => "PIN do odblokowania profilu fana",
        "enter_the_pin_created_for_this_fan" => {
            "Wpisz PIN ustawiony podczas konfiguracji tego profilu fana. To nie jest kod QR ani PIN telefonu."
        }
        "create_fan_unlock_pin" => "Utwórz PIN do profilu fana",
        "enter_4_6_digits_for_this_fan_profile" => {
            "Wpisz 4–6 cyfr, np. 2580. Ten PIN odblokowuje tylko profil fana w Virya Signal."
        }
        "this_show_has_no_ticket_pool" => "Ten koncert nie ma puli biletowej.",
        "ticket_pool_status_loading" => "SPRAWDZAM PULĘ…",
        "ticket_pool_temporarily_unavailable" => "Pula biletowa jest chwilowo niedostępna.",
        "could_not_save_city_message" => "Nie udało się zapisać miasta: {message}.",
        "new_message_not_sent_previous_code_still_valid_minutes" => {
            "Nowa wiadomość nie została wysłana, bo poprzedni kod jest nadal ważny. Spróbuj ponownie za około {minutes} min."
        }
        "could_not_refresh_orders_count_other_tickets_remain_available" => {
            "Nie udało się odświeżyć {count} zamówień. Pozostałe bilety są nadal dostępne."
        }
        "area_city_wroclaw" => "Wrocław",
        "area_city_poznan" => "Poznań",
        "area_city_gdansk" => "Gdańsk",
        "area_city_warsaw" => "Warszawa",
        "area_city_katowice" => "Katowice",
        "area_city_krakow" => "Kraków",
        "area_city_lodz" => "Łódź",
        "area_city_szczecin" => "Szczecin",
        "area_city_lublin" => "Lublin",
        "area_city_rzeszow" => "Rzeszów",
        "area_city_bialystok" => "Białystok",
        "area_city_torun" => "Toruń",
        "area_region_lower_silesia" => "Dolny Śląsk",
        "area_region_greater_poland" => "Wielkopolska",
        "area_region_pomerania" => "Pomorze",
        "area_region_masovia" => "Mazowsze",
        "area_region_silesia" => "Śląsk",
        "area_region_lesser_poland" => "Małopolska",
        "area_region_lodz" => "Łódzkie",
        "area_region_west_pomerania" => "Zachodniopomorskie",
        "area_region_lublin" => "Lubelskie",
        "area_region_subcarpathia" => "Podkarpackie",
        "area_region_podlasie" => "Podlaskie",
        "area_region_kuyavia_pomerania" => "Kujawsko-Pomorskie",
        "area_clue_wroclaw" => "Sygnał zbiera się gdzieś pomiędzy betonem, wodą i hałasem.",
        "area_clue_poznan" => "Idź za złotym sygnałem. Zostaw oczywistą trasę za sobą.",
        "area_clue_gdansk" => "Szukaj echa tam, gdzie stal spotyka sól.",
        "area_clue_warsaw" => "Najgłośniejsze miasto ukrywa najcichszą transmisję.",
        "area_clue_katowice" => "Przemysłowy puls czeka tuż pod powierzchnią.",
        "area_clue_krakow" => "Stary kamień. Nowy hałas. Jedna linia zamknięta w środku.",
        "area_clue_lodz" => "Idź za nicią przez cegłę, tory i miasto wymyślone na nowo.",
        "area_clue_szczecin" => "Sygnał płynie w głąb lądu od wody ułożonej jak labirynt.",
        "area_clue_lublin" => "Słuchaj tam, gdzie stare bramy niosą nową częstotliwość.",
        "area_clue_rzeszow" => "Południowy puls ukrywa się między ruchem a otwartym niebem.",
        "area_clue_bialystok" => "Na skraju lasu cichy sygnał dociera najdalej.",
        "area_clue_torun" => "Spójrz w górę, potem sprowadź orbitę z powrotem na ulicę.",
        "audience_intelligence" => "Audience Intelligence",
        "fan_360_summary" => "Fan 360 · agregaty",
        "ticket_buyers" => "kupili bilet",
        "concert_attendees" => "byli na koncercie",
        "synesthesia_participants" => "Synesthesia",
        "qualified_referrals" => "polecenia qualified",
        "ticket_revenue" => "przychód biletowy",
        "after_refunds" => "po refundach",
        "paid_orders_count" => "{0} płatnych zamówień",
        "direction_to_point" => "{arrow} {direction}",
        "direction_north" => "północ",
        "direction_northeast" => "północny wschód",
        "direction_east" => "wschód",
        "direction_southeast" => "południowy wschód",
        "direction_south" => "południe",
        "direction_southwest" => "południowy zachód",
        "direction_west" => "zachód",
        "direction_northwest" => "północny zachód",
        "signal_live_now" => "SYGNAŁ TRWA TERAZ",
        "signal_afterglow" => "PO SYGNALE",
        "open_wallet_now" => "OTWÓRZ BILET / PASS",
        "open_live_signal" => "OTWÓRZ KONCERT",
        "share_post_show_feedback" => "ZOSTAW ECHO PO KONCERCIE",
        "get_ticket_now" => "ZDOBĄDŹ BILET",
        "follow_this_signal" => "OBSERWUJ TEN SYGNAŁ",
        "signal_live_note" => "Jesteś w oknie koncertu — najważniejsze rzeczy są pod ręką.",
        "signal_afterglow_note" => {
            "Koncert właśnie wybrzmiał. Zostaw krótkie anonimowe echo, zanim wrażenie zniknie."
        }
        "unsupported_signal_snapshot_version" => {
            "Ten snapshot Sygnału używa nieobsługiwanej wersji schematu {}. Zaktualizuj aplikację."
        }
        "unsupported_staff_snapshot_version" => {
            "Ten snapshot staff używa nieobsługiwanej wersji schematu {}. Zaktualizuj aplikację."
        }
        "signal_snapshot_updated" => "Aktualizacja: {}",
        "synesthesia_completed_in_minutes" => "Ukończono w około {} min",
        "reward_entry_confirmed" => "Udział w nagrodzie potwierdzony",
        "doors_open_at" => "Otwarcie bram: {}",
        "event_ends_at" => "Koniec: {}",
        "entry_ready" => "Wejście gotowe",
        "following_event" => "Obserwujesz",
        "tickets_on_sale" => "Bilety w sprzedaży",
        "paid_orders" => "opłacone zamówienia",
        "pending_referrals" => "oczekujące polecenia",
        "passes_issued" => "wydane passy",
        "could_not_refresh_orders_cached_orders_available" => {
            "Nie udało się odświeżyć {} zamówień; {} kopii offline jest nadal dostępnych."
        }
        "autopilot_control" => "Autonomiczne operacje",
        "autopilot_runtime_disabled" => {
            "Runtime Autopilota jest globalnie wyłączony. Polityki można przygotować, ale żadna decyzja nie zostanie wykonana."
        }
        "autopilot_nothing_needs_you" => {
            "Nic nie wymaga Twojej decyzji — ViryaOS obsługuje bieżące operacje."
        }
        "autopilot_needs_you" => "Wymaga Ciebie",
        "autopilot_recent_actions" => "Ostatnie działania",
        "autopilot_measured_effects" => "Zmierzony efekt",
        "autopilot_effect_improved" => "poprawa",
        "autopilot_effect_neutral" => "bez istotnej zmiany",
        "autopilot_effect_worsened" => "pogorszenie",
        "autopilot_actions_24h" => "wykonane 24h",
        "autopilot_queue" => "w kolejce",
        "autopilot_failed_24h" => "błędy 24h",
        "autopilot_authority" => "Poziom autonomii",
        "autopilot_financial_guardrails" => "Limity finansowe",
        "autopilot_off" => "OFF",
        "autopilot_observe" => "OBSERWUJ",
        "autopilot_recommend" => "REKOMENDUJ",
        "autopilot_approval" => "AKCEPTACJA",
        "autopilot_auto" => "AUTO",
        "autopilot_guarded" => "AUTO WSTRZYMANE",
        "autopilot_expires" => "akceptacja wygasa",
        "autopilot_executor_confirmed" => "executor potwierdził",
        "autopilot_executor_failed" => "executor błąd",
        "autopilot_release_ledger" => "Stan wdrożeń",
        "autopilot_release_drift" => "ROZJAZD",
        "autopilot_release_sync" => "SYNC",
        "autopilot_n8n_executors" => "executory n8n",
        "autopilot_executor_guards" => "blokady executora",
        "autopilot_release_missing" => "brakuje",
        "autopilot_release_stale" => "NIEAKTUALNE",
        "autopilot_release_production" => "produkcja",
        "autopilot_rum_24h" => "Realna wydajność użytkowników · 24h",
        "autopilot_samples" => "próbek",
        "autopilot_assign" => "PRZYPISZ",
        "autopilot_assign_to" => "Przypisz zadanie",
        "autopilot_approve" => "AKCEPTUJ",
        "autopilot_cancel" => "ANULUJ",
        "autopilot_chief" => "Szef operacyjny",
        "autopilot_time_saved" => "czas zdjęty z zespołu",
        "autopilot_improved_7d" => "poprawione 7d",
        "autopilot_deadline_radar" => "Radar terminów",
        "autopilot_attention_approval" => "Wygasa akceptacja",
        "autopilot_attention_opportunity" => "Termin okazji",
        "autopilot_attention_funding" => "Termin finansowania",
        "autopilot_urgency_overdue" => "PO TERMINIE",
        "autopilot_urgency_critical" => "PILNE",
        "autopilot_urgency_today" => "DZISIAJ",
        "autopilot_urgency_soon" => "WKRÓTCE",
        "autopilot_urgency_upcoming" => "NADCHODZI",
        "autopilot_opportunities" => "Najlepsze okazje",
        "autopilot_show_tasks" => "Rzeczy koncertowe do dopięcia",
        "autopilot_manual_steps" => "Ręcznie: {}",
        "autopilot_beacon_discovery_detail" => "Beacon · znajdź {} lokalnych latarni",
        "autopilot_invalid_action" => "Nieprawidłowa akcja Autopilota",
        "autopilot_funding_package_detail" => "Przygotowanie pakietu wniosku",
        "autopilot_funding_submit_detail" => "Złożenie gotowego wniosku",
        "database_runtime" => "PostgreSQL runtime",
        "async_io" => "async I/O",
        "area_runtime" => "AREA — źródło stanu",
        "area_credits" => "kredyty",
        "area_vouchers" => "vouchery",
        "area_ticket_rewards" => "nagrody biletowe",
        "area_legacy_imported" => "zaimportowane legacy",
        "stale_voucher_leases" => "stare lease voucherów",
        "stale_ticket_leases" => "stare lease biletów",
        "push_notifications" => "Powiadomienia push",
        "push_notifications_hint" => {
            "Koncerty w pobliżu, bilety i ważne wiadomości Signal mogą pojawić się bezpośrednio na tym telefonie."
        }
        "push_notifications_on" => "Powiadomienia są aktywne na tym urządzeniu.",
        "push_notifications_off" => "Powiadomienia są wyłączone na tym urządzeniu.",
        "push_notifications_blocked" => {
            "Android blokuje powiadomienia. Włącz je dla Virya Signal w ustawieniach telefonu."
        }
        "push_notifications_waiting_backend" => {
            "Kanał push nie jest jeszcze aktywny po stronie VIRYA OS."
        }
        "push_notifications_degraded" => {
            "Nie udało się potwierdzić synchronizacji push. Spróbuj ponownie bez zmiany ustawienia."
        }
        "enable_push_notifications" => "WŁĄCZ POWIADOMIENIA",
        "disable_push_notifications" => "WYŁĄCZ POWIADOMIENIA",
        "syncing_push_notifications" => "SYNCHRONIZUJĘ…",
        _ => key,
    }
}
