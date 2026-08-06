pub(super) fn text(key: &'static str) -> &'static str {
    match key {
        "sygna" => "← SIGNAL",
        "jestes_w_staffie" => "ARE YOU ON THE STAFF?",
        "podaj_haso_staff_uzywane_w_panelu_virya" => {
            "Enter the staff password used in the Virya panel."
        }
        "strefa" => "The ",
        "zespou" => "team zone.",
        "dostep_do_bramki_sprzedazy_i_obsugi_koncertu_jest" => {
            "Gate, sales and show operations access is separate from the fan account."
        }
        "weryfikacja_staff" => "STAFF VERIFICATION",
        "haso_panelu_virya" => "Virya panel password",
        "uzyj_tego_samego_hasa_co_w_qr_bramce" => {
            "Use the same password as in QR, Gate and Control Center. After verification, the app will show the local PIN or device pairing."
        }
        "haso_staff" => "Staff password",
        "sprawdzam" => "CHECKING…",
        "otworz_strefe_staff" => "OPEN STAFF ZONE",
        "haso_jest_sprawdzane_po_stronie_virya_music_nie" => {
            "The password is verified by virya.music and is not stored in the app."
        }
        "nie_udao_sie_odczytac_sejfu_staff" => "FAILED TO READ THE STAFF VAULT",
        "sprawdzam_bezpieczny_sejf" => "CHECKING THE SECURE VAULT",
        "virya_staff" => "Virya staff",
        "pin_musi_miec_co_najmniej_4_znaki" => "The PIN must contain at least 4 characters.",
        "podaj_46_cyfrowy_pin_i_zeskanuj_albo_wklej" => {
            "Enter a 4–6 digit PIN and scan or paste the pairing code."
        }
        "podaj_46_cyfrowy_pin_i_poprawny_token_urzadzenia" => {
            "Enter a 4–6 digit PIN and a valid device token."
        }
        "kod_zeskanowany" => "Code scanned",
        "wpisz_pin_ponizej_i_kliknij_sparuj" => "Enter the PIN below and tap PAIR.",
        "acze" => "CONNECTING…",
        "zeskanuj_kod_qr" => "SCAN QR CODE",
        "kod_pokazany_w_panelu_virya" => "Code shown in the Virya panel",
        "albo" => "OR",
        "kod_parowania" => "Pairing code",
        "utworz_pin_do_odblokowania" => "Create an unlock PIN",
        "wpisz_46_cyfr_np_2580_to_pin_tylko" => {
            "Enter 4–6 digits, for example 2580. This PIN unlocks Virya Signal only — it is not the QR code or your phone PIN."
        }
        "np_2580" => "e.g. 2580",
        "ukryj_ustawienia_reczne" => "HIDE MANUAL SETTINGS",
        "ustawienia_zaawansowane" => "ADVANCED SETTINGS",
        "nazwa_urzadzenia_osoby" => "Device / person name",
        "token_urzadzenia" => "Device token",
        "zapisz_recznie" => "SAVE MANUALLY",
        "pin_do_odblokowania_aplikacji" => "App unlock PIN",
        "wpisz_pin_ustawiony_podczas_parowania_tego_urzadzenia" => {
            "Enter the PIN created when this device was paired."
        }
        "twoj_pin" => "Your PIN",
        "odblokuj" => "UNLOCK",
        "otworz_menu" => "Open menu",
        "zamknij_i_zablokuj_panel" => "Close and lock panel",
        "znizki" => "Discounts",
        "kody_qr" => "QR codes",
        "ustawienia" => "Settings",
        "start" => "Home",
        "sygna_2" => "Signal",
        "skan" => "Scan",
        "bilety" => "Tickets",
        "koncerty" => "Shows",
        "sklep" => "Store",
        "profil" => "Profile",
        "gra_area" => "AREA game",
        "strefa_staff" => "Staff zone",
        "zamknij_i_zablokuj_sygna" => "Close and lock Signal",
        "koncerty_2" => "shows",
        "aktywne_qr" => "active QR",
        "check_iny" => "check-ins",
        "brak_nadchodzacych_koncertow" => "No upcoming shows",
        "kiedy_pojawi_sie_nowe_wydarzenie_zobaczysz_je_tutaj" => "New events will appear here.",
        "widok_tylko_dla_ownera" => "Owner-only view",
        "statystyki_zgod_wzrostu_i_miast_sa_dostepne_wyacznie" => {
            "Consent, growth and city statistics are available to the owner only."
        }
        "dane_sa_agregowane_w_crowdrelay_i_nie_zawieraja" => {
            "Data is aggregated in CrowdRelay and contains no email addresses or fan identifiers."
        }
        "odswiezam" => "REFRESHING…",
        "odswiez" => "REFRESH",
        "brak_snapshotu_sygnau" => "No Signal snapshot",
        "odswiez_dane_jezeli_backend_jest_jeszcze_w_trakcie" => {
            "Refresh the data. If the backend is still being deployed, the panel will show a safe error instead of an empty screen."
        }
        "snapshot_czesciowy_niedostepne_zroda_value" => {
            "Partial snapshot. Unavailable sources: {}."
        }
        "brak_agregatu_miast" => "No city aggregate",
        "sygna_nie_ma_jeszcze_potwierdzonych_danych_miejskich_albo" => {
            "Signal has no confirmed city data yet, or the source is temporarily unavailable."
        }
        "aktywni" => "active",
        "zgody_marketingowe" => "marketing consents",
        "nowi_30_dni" => "new / 30 days",
        "potwierdzonych_sposrod_aktywnych_i_oczekujacych" => "confirmed among active and pending",
        "wszyscy" => "all",
        "oczekujacy" => "pending",
        "wypisani" => "unsubscribed",
        "wyciszeni" => "muted",
        "powiadomienia_w_poblizu" => "nearby notifications",
        "aktywnosc" => "Activity",
        "30_dni_caosc" => "30 days / total",
        "nowi_7_dni" => "new / 7 days",
        "polecenia" => "referrals",
        "zainteresowania_koncertami" => "show interests",
        "powiadomienia_nearby" => "nearby notifications",
        "miasta_do_moderacji" => "cities awaiting moderation",
        "najsilniejsze_miasta" => "Strongest cities",
        "snapshot_generated_at_dane_wyacznie_zagregowane" => {
            "Snapshot: {generated_at}. Aggregated data only."
        }
        "najpierw_wybierz_koncert" => "Select a show first.",
        "wybierz_koncert" => "Select a show.",
        "snapshot_gotowy_value_trwaych_biletow" => "Snapshot ready: {} durable tickets.",
        "sync_value_zapisane_value_konfliktow_value_nadal_czeka" => {
            "Sync: {} saved, {} conflicts, {} still pending."
        }
        "dane_koncertu_usuniete_z_urzadzenia" => "Show data removed from the device.",
        "koncert" => "Show",
        "aduje_koncerty" => "Loading shows…",
        "wybierz_wydarzenie" => "Select an event",
        "bramka_pracuje_lokalnie" => "Gate works locally",
        "odpornosc_na_brak_lte" => "Works without LTE",
        "pobierz_bezpieczny_snapshot_przed_otwarciem_bram" => {
            "Download a secure snapshot before opening the gates."
        }
        "przygotuj_offline" => "PREPARE OFFLINE",
        "synchronizuj" => "SYNC",
        "wyczysc" => "CLEAR",
        "weryfikuje" => "VERIFYING…",
        "skanuj_lokalnie" => "SCAN LOCALLY",
        "uruchom_aparat" => "OPEN CAMERA",
        "wyacznie_trway_qr_biletu_t1" => "Durable t1 ticket QR only",
        "qr_biletu_lub_wejsciowki" => "Ticket or admission-pass QR",
        "kod_qr_lub_numer_wejsciowki" => "QR code or admission-pass number",
        "sprawdz" => "CHECK",
        "wybierz_koncert_i_podaj_e_mail_fana" => "Select a show and enter the fan email.",
        "podaj_public_reference_wejsciowki" => "Enter the admission pass public reference.",
        "wejsciowka_zostaa_uniewazniona" => "The admission pass has been revoked.",
        "wybierz_koncert_2" => "Select a show",
        "sprzedane" => "sold",
        "w_trakcie" => "in checkout",
        "dostepne" => "available",
        "zwroty_value" => "refunds: {}",
        "numer_wejsciowki_to_bezpieczny_publiczny_identyfikator_np_vry" => {
            "The admission-pass number is a safe public identifier, e.g. VRY-... It is not a QR token or a private order token."
        }
        "wydaj_wejsciowke" => "ISSUE PASS",
        "numer_wejsciowki_np_vry" => "Admission-pass number, e.g. VRY-…",
        "uniewaznij" => "REVOKE",
        "podaj_kod_i_numer_sprzedazy" => "Enter the code and sale number.",
        "kod_znizkowy" => "Discount code",
        "numer_sprzedazy" => "Sale number",
        "zrealizuj_kupon" => "REDEEM COUPON",
        "uzycie_value_value" => "Usage {}/{}",
        "wejscie_gowne" => "Main entrance",
        "podaj_poprawny_poczatek_waznosci" => "Enter a valid start date.",
        "podaj_poprawny_koniec_waznosci" => "Enter a valid end date.",
        "limit_musi_byc_dodatnia_liczba" => "The limit must be a positive number.",
        "koniec_kampanii_musi_byc_pozniej_niz_poczatek" => {
            "The campaign end must be after its start."
        }
        "wybierz_koncert_i_nazwij_kampanie" => "Select a show and name the campaign.",
        "kampania_qr_utworzona" => "QR campaign created.",
        "aduje_kampanie" => "Loading campaigns…",
        "nazwa_punktu_kampanii" => "Point / campaign name",
        "wazna_od" => "Valid from",
        "wazna_do" => "Valid until",
        "limit_check_inow_opcjonalnie" => "Check-in limit (optional)",
        "utworz_kampanie" => "CREATE CAMPAIGN",
        "kampania_zostaa_wyaczona" => "The campaign has been disabled.",
        "wyacz_kampanie" => "DISABLE CAMPAIGN",
        "limit_value" => "limit {v}",
        "bez_limitu" => "no limit",
        "poaczenie" => "Connection",
        "uprawnienia" => "Permissions",
        "odswiez_wszystkie_dane" => "Refresh all data",
        "zablokuj_panel" => "Lock panel",
        "usun_profil_operatora" => "Remove operator profile",
        "token_operatora_przechowuje_zaszyfrowany_sejf_stronghold_warstwa_webview" => {
            "The operator token is stored in an encrypted Stronghold vault. The WebView layer never reads it."
        }
        "odswiez_2" => "Refresh",
        "cockpit_dziaa_czesciowo_niedostepne_value" => {
            "Cockpit is partially available. Unavailable: {}."
        }
        "brak_kodu_bedu" => "no error code",
        "proba_value_value" => "attempt {}/{}",
        "retry_by_juz_wczesniej_przyjety" => "The retry had already been accepted.",
        "wpis_wroci_do_kolejki" => "The item returned to the queue.",
        "jezyk" => "Language",
        "jezyk_aplikacji" => "App language",
        "zmiana_jezyka_przeaduje_interfejs_dane_i_sesja_pozostana" => {
            "Changing the language reloads the interface. Your data and session remain unchanged."
        }
        "polski" => "Polish",
        "angielski" => "English",
        "nie_udao_sie_odczytac_profilu_fana" => "FAILED TO READ THE FAN PROFILE",
        "sprawdzam_twoj_sygna" => "CHECKING YOUR SIGNAL",
        "profil_pozostaje_nietkniety" => "Your profile remains untouched.",
        "aplikacja_nie_przejdzie_do_rejestracji_ani_parowania_dopoki" => {
            "The app will not continue to signup or pairing until it confirms the encrypted vault state on this device."
        }
        "sprobuj_ponownie" => "TRY AGAIN",
        "podaj_e_mail_uzyty_przy_zapisie_do_sygnau" => "Enter the email used to join Signal.",
        "wklej_kod_cay_link_albo_zeskanuj_qr_z" => {
            "Paste the code or full link, or scan the QR from the message."
        }
        "ustaw_lokalny_pin_majacy_co_najmniej_4_znaki" => {
            "Create a local PIN with at least 4 characters."
        }
        "zgoda_marketingowa_jest_wymagana_do_doaczenia_do_sygnau" => {
            "Marketing consent is required to join Signal."
        }
        "nie_udao_sie_zapisac_miasta_message" => "Could not save the city: {message}",
        "wybierz_miasto_albo_wpisz_wasne" => "Select a city or enter your own.",
        "wysalismy_bezpieczny_link_dostepu_zeskanuj_qr_albo_wklej" => {
            "We sent a secure access link. Scan the QR or paste the code from the message."
        }
        "wysalismy_kod_potwierdzajacy_zeskanuj_qr_albo_wklej_kod" => {
            "We sent a confirmation code. Scan the QR or paste the code from the message."
        }
        "nowa_wiadomosc_nie_zostaa_wysana_bo_poprzedni_kod" => {
            "A new message was not sent because the previous code is still valid. Use the previous message or try again in about {minutes} min."
        }
        "zgoszenie_zostao_przyjete_sprawdz_skrzynke_i_spam_jesli" => {
            "The request was accepted. Check your inbox and spam; if the message is missing, try again later."
        }
        "podaj_e_mail_uzyty_w_virya_signal" => "Enter the email used in Virya Signal.",
        "jesli_ten_e_mail_jest_zapisany_w_virya" => {
            "If this email is registered in Virya Signal, we sent a fresh login link with a QR code. After opening it, set a new PIN for this device."
        }
        "qr_zeskanowany_uzupenij_e_mail_i_lokalny_pin" => {
            "QR scanned. Enter your email and local PIN."
        }
        "koncerty_bilety" => "Shows, tickets",
        "i_nagrody" => "and rewards.",
        "doacz_w_3_krokach" => "Join in 3 steps:",
        "jak_doaczyc" => "How to join",
        "podaj_e_mail_i_miasto" => " Enter your email and city",
        "potwierdz_kod_z_wiadomosci" => " Confirm the code from the message",
        "odkrywaj_koncerty_blisko_ciebie" => " Discover shows near you",
        "co_daje_virya_signal" => "What Virya Signal gives you",
        "koncerty_blisko_ciebie" => " shows near you",
        "bilety_i_qr_w_telefonie" => " tickets and QR codes on your phone",
        "nagrody_za_proste_akcje" => " rewards for simple actions",
        "zaczynam" => "GET STARTED",
        "mam_kod" => "I HAVE A CODE",
        "e_mail" => "Email",
        "imie_nazwa_opcjonalnie" => "Name (optional)",
        "najszybciej_zeskanuj_qr_z_maila" => "Fastest: scan the QR from the email.",
        "mozesz_tez_wkleic_cay_link_albo_64_znakowy" => {
            "You can also paste the full link or the 64-character code. The app will extract the correct token."
        }
        "link_lub_kod_z_e_maila" => "Email link or code",
        "wklej_link_kod_albo_uzyj_qr" => "Paste a link or code, or use QR",
        "skanuj_qr" => "SCAN QR",
        "albo_przytrzymaj_pole_wyzej_i_wybierz_wklej" => "or hold the field above and choose Paste",
        "lokalny_pin" => "Local PIN",
        "pin_szyfruje_profil_tylko_na_tym_urzadzeniu_nie" => {
            "The PIN encrypts your profile on this device only. It is never sent to CrowdRelay."
        }
        "potwierdz_i_wejdz" => "CONFIRM AND ENTER",
        "mam_juz_konto_wyslij_link_logowania" => "I ALREADY HAVE AN ACCOUNT — SEND LOGIN LINK",
        "nie_ma_wiadomosci_sprawdz_spam_po_15_minutach" => {
            "No message? Check spam. After 15 minutes, return to GET STARTED and request another code."
        }
        "miejscowosc" => "City",
        "np_bielawa" => "e.g. Bielawa",
        "wojewodztwo_region_opcjonalnie" => "Province / region (optional)",
        "dolnoslaskie" => "Lower Silesia",
        "wpisz_miejscowosc_recznie_dopasujemy_ja_do_mapy_sygnau" => {
            "Enter your city manually — we will match it to the Signal map."
        }
        "powiadamiaj_mnie_o_koncertach_w_poblizu" => "Notify me about nearby shows",
        "kod_polecajacy_opcjonalnie" => "Referral code (optional)",
        "chce_otrzymywac_informacje_o_koncertach_premierach_i_nagrodach" => {
            "I want to receive information about Virya shows, releases and rewards."
        }
        "doacz_do_sygnau" => "JOIN SIGNAL",
        "otworz_moj_sygna" => "OPEN MY SIGNAL",
        "nie_pamietam_pin_u_zaloguj_ponownie" => "I FORGOT MY PIN / SIGN IN AGAIN",
        "odzyskiwanie_dostepu" => "ACCESS RECOVERY",
        "ustaw_nowy_pin" => "Create a new PIN",
        "podaj_e_mail_wyslij_swiezy_link_a_potem" => {
            "Enter your email, request a fresh link, then scan the QR or paste the code."
        }
        "wyslij_link_logowania" => "SEND LOGIN LINK",
        "wklej_link_lub_kod" => "Paste link or code",
        "albo_przytrzymaj_pole_i_wybierz_wklej" => "or hold the field and choose Paste",
        "nowy_lokalny_pin" => "New local PIN",
        "potwierdz_i_ustaw_nowy_pin" => "CONFIRM AND SET NEW PIN",
        "wroc_do_logowania_pin_em" => "BACK TO PIN LOGIN",
        "kod_value" => "Code: {}",
        "adowanie_sygnau" => "Loading Signal…",
        "losy" => "entries",
        "kupony" => "coupons",
        "losowanie_value" => "Draw {}",
        "dowod" => "PROOF ↗",
        "merch" => "Merch",
        "produkty_i_zestawy_korzystaja_z_tego_samego_stanu" => {
            "Products and bundles use the same inventory as the online store. Payment opens secure Stripe Checkout and the app never stores card data."
        }
        "sklep_jest_chwilowo_niedostepny" => "The store is temporarily unavailable",
        "pozostae_czesci_sygnau_dziaaja_normalnie_sprobuj_ponownie_za" => {
            "The rest of Signal is working normally. Try again in a moment."
        }
        "odswiez_merch" => "REFRESH MERCH",
        "otworz_peny_sklep" => "OPEN FULL STORE ↗",
        "zestawy" => "BUNDLES",
        "bundle_ze_sklepu_online" => "Bundles from the online store",
        "do_30" => "UP TO −30%",
        "zestawy_sa_teraz_niedostepne_w_live_inventory" => {
            "Bundles are currently unavailable in live inventory."
        }
        "zobacz_zestawy" => "VIEW BUNDLES ↗",
        "ostatnie_sztuki" => "LOW STOCK",
        "dostepny" => "AVAILABLE",
        "brak_na_stanie" => "OUT OF STOCK",
        "sprawdz_ponownie" => "CHECK AGAIN",
        "kup_w_sklepie" => "BUY IN STORE ↗",
        "zestawy_doczytuja_sie_niezaleznie_od_produktow" => {
            "Bundles load independently from products."
        }
        "pojedyncze_produkty" => "INDIVIDUAL PRODUCTS",
        "wybierz_swoj_merch" => "Choose your merch",
        "przedsprzedaz" => "PRE-ORDER",
        "nie_udao_sie_pobrac_stanu_sklepu" => "Could not load store status",
        "koncerty_bilety_i_profil_pozostaja_dostepne" => {
            "Shows, tickets and profile remain available."
        }
        "brak_koncertow_w_kalendarzu" => "No shows in the calendar",
        "kiedy_pojawi_sie_nowy_event_bedzie_tutaj" => "New events will appear here.",
        "koncert_zapisany_w_twoim_sygnale" => "Show saved to your Signal.",
        "zapisuje" => "SAVING…",
        "mam_to" => "✓ SAVED",
        "interesuje_mnie" => "+ INTERESTED",
        "kup_bilet" => "BUY TICKET",
        "wroc_do_koncertow" => "← BACK TO SHOWS",
        "nie_udao_sie_sprawdzic_sprzedazy" => "Could not check ticket sales",
        "brak_wasnej_puli_virya" => "No Virya ticket pool",
        "mozesz_przejsc_do_strony_koncertu_lub_sprzedazy_prowadzonej" => {
            "You can open the show page or the organiser’s ticket sale."
        }
        "sprawdz_bilety" => "CHECK TICKETS ↗",
        "sprzedaz_rozpocznie_sie_wkrotce" => "Ticket sales will open soon.",
        "sprzedaz_online_zostaa_zakonczona" => "Online sales have ended.",
        "ta_pula_biletow_jest_wyprzedana" => "This ticket pool is sold out.",
        "sprzedaz_jest_chwilowo_wyaczona" => "Ticket sales are temporarily disabled.",
        "ten_koncert_nie_jest_obecnie_dostepny_w_sprzedazy" => {
            "This show is not currently on sale."
        }
        "bilety_nie_sa_teraz_dostepne" => "Tickets are not available right now.",
        "wybierz_bilety_miejsca_zostana_zarezerwowane_na_czas_patnosci" => {
            "Select tickets. Places will be reserved while you complete payment."
        }
        "w_patnosci" => "in checkout",
        "sprawdz_strone_koncertu" => "Open the show page",
        "jezeli_organizator_prowadzi_osobna_sprzedaz_znajdziesz_ja_pod" => {
            "If the organiser runs a separate ticket sale, you will find it under this button."
        }
        "wybierz_co_najmniej_jeden_bilet" => "Select at least one ticket.",
        "zamowienie_value_zapisane_dokoncz_bezpieczna_patnosc_stripe" => {
            "Order {} saved. Complete the secure Stripe payment."
        }
        "otworzono_patnosc_dla_zamowienia_value" => "Payment opened for order {}.",
        "dostepne_value" => "Available: {}",
        "liczba_biletow" => "Ticket quantity",
        "zmniejsz_liczbe_biletow" => "Decrease ticket quantity",
        "zwieksz_liczbe_biletow" => "Increase ticket quantity",
        "imie_i_nazwisko_na_zamowieniu_opcjonalnie" => "Name on the order (optional)",
        "bilety_i_potwierdzenie_trafia_na_value" => "Tickets and confirmation will be sent to {}",
        "bilety_trafia_na_e_mail_konta_fana" => "Tickets will be sent to the fan account email.",
        "faktura_peny_formularz" => "INVOICE / FULL FORM ↗",
        "wybrane_bilety" => "Selected tickets",
        "razem_brutto" => "Gross total",
        "rezerwuje" => "RESERVING…",
        "zamowienie_zapisane" => "ORDER SAVED",
        "przejdz_do_patnosci_stripe" => "CONTINUE TO STRIPE PAYMENT",
        "otworz_patnosc_ponownie" => "REOPEN PAYMENT ↗",
        "dane_karty_nie_trafiaja_do_virya_signal_patnosc" => {
            "Card details never reach Virya Signal. Payment opens in secure Stripe Checkout."
        }
        "otworz_mape_i_zacznij" => "OPEN MAP AND START",
        "odswiez_progres" => "Refresh progress",
        "otworz_area" => "OPEN AREA",
        "podaj_identyfikator_zamowienia_i_prywatny_token" => {
            "Enter the order ID and private token."
        }
        "bilety_zapisane_w_portfelu" => "Tickets saved to the wallet.",
        "wklej_token_wejsciowki" => "Paste the admission-pass token.",
        "wejsciowka_przypisana_do_urzadzenia" => "Admission pass assigned to this device.",
        "pokaz_qr_na_wejscie" => "SHOW ENTRY QR",
        "token_z_wiadomosci" => "Token from the message",
        "odbierz_wejsciowke" => "CLAIM ADMISSION PASS",
        "dodaj_istniejace_zamowienie" => "Add an existing order",
        "uuid_zamowienia" => "Order UUID",
        "prywatny_checkout_token" => "Private checkout token",
        "dodaj_do_portfela" => "ADD TO WALLET",
        "wysalismy_ponownie_portfel_na_e_mail" => "We resent the wallet by email.",
        "wysyam" => "Sending…",
        "wyslij_bilety_ponownie_na_e_mail" => "Resend tickets by email",
        "generuje" => "GENERATING…",
        "ukryj_qr" => "HIDE QR",
        "pokaz_qr" => "SHOW QR",
        "qr_niedostepny" => "QR UNAVAILABLE",
        "qr_wazny_do_value" => "QR valid until {}",
        "wazny_do_value" => "valid until {}",
        "moj_profil" => "MY PROFILE",
        "ustawienia_sygnau" => "Signal settings",
        "fan_viryi" => "Virya fan",
        "zamowienia" => "orders",
        "wejsciowki" => "admission passes",
        "odswiezam_2" => "Refreshing…",
        "odswiez_dane" => "Refresh data",
        "zablokuj_aplikacje" => "Lock app",
        "usun_profil_i_bilety_z_urzadzenia" => "Remove profile and tickets from device",
        "sesja_fana_wejsciowka_oraz_prywatne_tokeny_portfela_sa" => {
            "The fan session, admission pass and private wallet tokens are stored in a separate encrypted Stronghold vault."
        }
        "feedback_powinien_miec_od_8_do_2000_znakow" => {
            "Feedback must contain between 8 and 2000 characters."
        }
        "feedback_zosta_wysany_anonimowo_dzieki" => "Feedback was sent anonymously. Thank you!",
        "anonimowy_feedback" => "ANONYMOUS FEEDBACK",
        "powiedz_nam_co_poprawic" => "Tell us what to improve",
        "aplikacja_wysya_tylko_kategorie_i_tresc_bez_e" => {
            "The app sends only the category and message — no email, name, session token or profile identifier. Hosting may retain standard technical connection logs."
        }
        "kategoria" => "Category",
        "pomys" => "Idea",
        "bad" => "Bug",
        "koncerty_i_bilety" => "Shows and tickets",
        "inne" => "Other",
        "tresc" => "Message",
        "napisz_wprost_co_dziaa_zle_albo_czego_brakuje" => {
            "Tell us directly what is broken or missing…"
        }
        "wysyam_2" => "SENDING…",
        "wyslij_anonimowo" => "SEND ANONYMOUSLY",
        "adowanie" => "Loading",
        "feedback_zosta_wysany" => "feedback was sent",
        "nie_udao_sie_odswiezyc_value_zamowien_pozostae_bilety" => {
            "Could not refresh {} orders. The remaining tickets are available."
        }
        "szczegoy_wkrotce" => "Details coming soon",
        "miejsce_wkrotce" => "venue coming soon",
        "skaner_nie_zwroci_kodu" => "The scanner returned no code.",
        "odpowiedz_serwera_ma_nieoczekiwany_format" => {
            "The server response has an unexpected format."
        }
        "bad_odczytu_odpowiedzi_raw" => "Response decoding error: {raw}",
        "nieznany_bad_aplikacji" => "Unknown application error",
        "sparuj" => "Pair",
        "urzadzenie" => "device.",
        "bez_przepisywania_api_roli_i_dugiego_sekretu" => {
            "No retyping the API, role or long secret."
        }
        "sparuj_2" => "PAIR",
        "profil_operatora_jest_zaszyfrowany_lokalnie" => {
            "The operator profile is encrypted locally."
        }
        "dzisiaj_pod_kontrola" => "Today under control",
        "nastepny_koncert" => "NEXT SHOW",
        "nadchodzace" => "Upcoming",
        "spoecznosc_i_wzrost" => "Community and growth",
        "zbiorczy_obraz_sygnau_bez_danych_osobowych_fanow" => {
            "A combined Signal overview without fans’ personal data."
        }
        "zdrowie_bazy" => "DATABASE HEALTH",
        "skanuj_wejscie" => "Scan entry",
        "bilety_i_wejsciowki" => "Tickets and admission passes",
        "obrot_brutto" => "GROSS REVENUE",
        "ostatnie_zamowienia" => "Recent orders",
        "reczna_wejsciowka" => "Manual admission pass",
        "realizuj_znizke" => "Redeem a discount",
        "kupon_fanowski_kontrolowane_uzycie" => "fan coupon / controlled use",
        "kupon_zrealizowany" => "COUPON REDEEMED",
        "kampanie_qr" => "QR campaigns",
        "aktywne_i_historyczne" => "Active and historical",
        "kolejki_i_dostawy" => "Queues and deliveries",
        "martwe_dostawy" => "Dead deliveries",
        "martwy_outbox" => "Dead outbox",
        "brak_martwych_wpisow_tor_dostaw_jest_czysty" => {
            "No dead entries. The delivery pipeline is clean."
        }
        "twoj_profil_i_bilety_sa_zaszyfrowane_na_urzadzeniu" => {
            "Your profile and tickets are encrypted on this device."
        }
        "twoj_wpyw" => "YOUR IMPACT",
        "potwierdzonych_polecen" => "confirmed referrals",
        "twoje_kupony" => "Your coupons",
        "nagrody" => "Rewards",
        "aktywne_losowania" => "Active draws",
        "losow" => "ENTRIES",
        "gdzie_gramy" => "WHERE WE PLAY",
        "znajdz_punkt_w_swoim_miescie" => "Find a point in your city",
        "otworz_mape_wybierz_aktywny_punkt_i_przejdz_do" => {
            "Open the map, choose an active point and go there. You do not need to collect everything or travel across the country."
        }
        "poacz_portfel_przegladarkowy_z_kontem_na_stronie_area" => {
            "Connect your browser wallet to your AREA account to keep all progress."
        }
        "postep_kolekcji" => "COLLECTION PROGRESS",
        "odkryte_artefakty" => "Discovered artifacts",
        "mapa_pokazuje_aktywne_punkty_i_prowadzi_do_startu" => {
            "The map shows active points and gets you started. The exact location is revealed in the game only when needed."
        }
        "area_chwilowo_niedostepna" => "AREA is temporarily unavailable",
        "odswiez_dane_albo_otworz_pena_gre" => "Refresh the data or open the full game.",
        "bilety_i_wejscie" => "Tickets and entry",
        "wejsciowka_virya" => "VIRYA ADMISSION PASS",
        "wygraes_wejsciowke" => "DID YOU WIN AN ADMISSION PASS?",
        "przypisz_ja_do_telefonu" => "Assign it to your phone",
        "portfel_biletow" => "Ticket wallet",
        "pojedyncze_produkty_2" => "Individual products",
        "zestawy_2" => "Bundles",
        "retry" => "RETRY",
        "acze_2" => "CONNECTING",
        "online" => "ONLINE",
        "offline_on" => "OFFLINE ON",
        "offline_off" => "OFFLINE OFF",
        "active" => "ACTIVE",
        "closed" => "CLOSED",
        "owner" => "OWNER",
        "staff" => "STAFF",
        "device" => "DEVICE",
        "mobile_wallet" => "MOBILE WALLET",
        "virya_control" => "VIRYA CONTROL",
        "virya_signal" => "VIRYA SIGNAL",
        "virya_store" => "VIRYA STORE",
        "virya_area" => "VIRYA AREA",
        "virya_bilety" => "VIRYA // TICKETS",
        "value_biletow_value_oczekuje_value_konfliktow" => "{} tickets · {} pending · {} conflicts",
        "value_check_inow" => "{} check-ins",
        "value_proba_value_value" => "{} · attempt {}/{}",
        "moj_sygna" => "My Signal",
        "oczekujace" => "pending",
        "message_zamowienie_value_jest_zapisane_uzyj_przycisku_ponownego" => {
            "{message} Order {} is saved — use the reopen payment button."
        }
        "reward_credits_kredytow" => "{reward_credits} credits",
        "live_count_aktywne_punkty" => "{live_count} active points",
        "voucher_count_nagrody" => "{voucher_count} rewards",
        "community_percent_spoecznosci" => "{community_percent}% community",
        "wysalismy" => "sent",
        "uniewaznion" => "revoked",
        "sty" => "JAN",
        "lut" => "FEB",
        "mar" => "MAR",
        "kwi" => "APR",
        "maj" => "MAY",
        "cze" => "JUN",
        "lip" => "JUL",
        "sie" => "AUG",
        "wrz" => "SEP",
        "paz" => "OCT",
        "lis" => "NOV",
        "gru" => "DEC",
        "text" => "---",
        "value_aktywnych" => "{} active",
        "value_zestaw_merchu_virya" => "{} — Virya merch bundle",
        "value_merch_virya" => "{} — Virya merch",
        "value_koncert_virya" => "{} — Virya show",
        "natywny_most_aplikacji_nie_jest_dostepny" => "The native app bridge is unavailable.",
        "operacja_command_przekroczya_limit_czasu" => "Operation {command} timed out.",
        "modu_uprawnien_aparatu_nie_jest_dostepny_w_tej" => {
            "The camera permission module is unavailable in this app version."
        }
        "brak_dostepu_do_aparatu_wacz_aparat_dla_virya" => {
            "Camera access is denied. Enable Camera for Virya Signal in the app settings."
        }
        "skaner_kodu_qr" => "QR code scanner",
        "skanuj_kod_qr" => "SCAN QR CODE",
        "umiesc_kod_wewnatrz_ramki" => "Place the code inside the frame",
        "anuluj_skanowanie" => "← CANCEL SCANNING",
        "zamykam" => "CLOSING…",
        "skaner_jest_dostepny_tylko_w_aplikacji_ios_android" => {
            "The scanner is available only in the iOS/Android app."
        }
        "rodzaj" => "Type",
        "czas" => "Time",
        "operacja" => "Operation",
        "sciezka" => "Path",
        "poprzednie_uruchomienie_zakonczyo_sie_bedem" => "The previous launch ended with an error",
        "aplikacja_zatrzymaa_bad" => "The app caught an error",
        "nie_ukrywamy_awarii_skopiuj_raport_i_wyslij_go" => {
            "We do not hide failures. Copy the report and send it with a note about what you tapped."
        }
        "kopiuj_raport" => "COPY REPORT",
        "uruchom_ponownie" => "RESTART APP",
        "zamknij" => "CLOSE",
        "raport_skopiowany" => "Report copied.",
        "przytrzymaj_tekst_raportu_i_skopiuj_recznie" => {
            "Press and hold the report text and copy it manually."
        }
        "poprzednie_uruchomienie_przerwao_operacje_command" => {
            "The previous launch interrupted operation {command}."
        }
        "poprzednie_uruchomienie_zakonczyo_sie_bez_czystego_zamkniecia" => {
            "The previous launch ended without a clean shutdown."
        }
        "virya_signal_diagnostyka" => "VIRYA SIGNAL / DIAGNOSTICS",
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
        "native_invalid_label" => "Invalid {label}",
        "native_public_cache_too_large" => "The local public-data cache is too large",
        "native_missing_events_cache" => "The backend confirmed a non-existent event cache",
        "native_missing_cities_cache" => "The backend confirmed a non-existent city cache",
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
        "native_bundle_name_label" => "bundle name",
        "native_bundle_description_label" => "bundle description",
        "native_bundle_item_label" => "bundle item",
        "native_image_url_label" => "image address",
        "native_store_url_label" => "store address",
        "native_bundle_variant_label" => "bundle variant",
        "native_prepare_offline_event_first" => "Prepare the event for offline mode first",
        _ => key,
    }
}
