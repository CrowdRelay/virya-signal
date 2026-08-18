const THOMANN_QUAD_CORTEX_URL: &str = "https://www.thomann.pl/neural_dsp_quad_cortex.htm?offid=1&affid=4979&subid=signal&subid2=gear";
const THOMANN_HOME_URL: &str =
    "https://www.thomann.pl/?offid=1&affid=4979&subid=signal&subid2=shop";

#[derive(Clone, Copy)]
struct AffiliateGearCopy {
    section_aria: &'static str,
    eyebrow: &'static str,
    title: &'static str,
    intro: &'static str,
    used_live: &'static str,
    product_note: &'static str,
    product_cta: &'static str,
    general_title: &'static str,
    general_note: &'static str,
    general_cta: &'static str,
    disclosure: &'static str,
}

fn affiliate_gear_copy() -> AffiliateGearCopy {
    match i18n::current() {
        Language::Pl => AffiliateGearCopy {
            section_aria: "Sprzęt VIRYA i linki afiliacyjne Thomann",
            eyebrow: "SPRZĘT VIRYA",
            title: "Sprzęt, którego naprawdę używamy",
            intro: "Bez katalogu sponsorów. Tylko rzeczy, które faktycznie trafiają do naszego live rigu.",
            used_live: "UŻYWAMY NA ŻYWO",
            product_note: "Nasz główny procesor gitarowy i centrum live rigu.",
            product_cta: "SPRAWDŹ W THOMANN ↗",
            general_title: "I tak robisz zakupy w Thomannie?",
            general_note: "Zacznij przez VIRYA. Twój zakup może wesprzeć kolejne koncerty i projekty.",
            general_cta: "ZACZNIJ PRZEZ VIRYA ↗",
            disclosure: "Linki afiliacyjne. Zakup może przynieść VIRYA prowizję bez dodatkowych kosztów dla Ciebie.",
        },
        Language::En => AffiliateGearCopy {
            section_aria: "VIRYA gear and Thomann affiliate links",
            eyebrow: "VIRYA GEAR",
            title: "Gear we actually use",
            intro: "No sponsor catalogue. Just equipment that is genuinely part of our live rig.",
            used_live: "USED LIVE",
            product_note: "Our main guitar processor and the centre of the live rig.",
            product_cta: "VIEW AT THOMANN ↗",
            general_title: "Already shopping at Thomann?",
            general_note: "Start through VIRYA. Your purchase can help support future shows and projects.",
            general_cta: "START THROUGH VIRYA ↗",
            disclosure: "Affiliate links. A purchase may earn VIRYA a commission at no extra cost to you.",
        },
    }
}

#[component]
fn FanAffiliateGear(error: RwSignal<Option<String>>) -> impl IntoView {
    let copy = affiliate_gear_copy();

    view! {
        <section class="affiliate-gear" aria-label=copy.section_aria>
            <header class="affiliate-gear-heading">
                <div>
                    <p class="eyebrow">{copy.eyebrow}</p>
                    <h3>{copy.title}</h3>
                    <p>{copy.intro}</p>
                </div>
                <span class="affiliate-partner-pill">"THOMANN · AFFILIATE"</span>
            </header>

            <article class="affiliate-gear-card">
                <div class="affiliate-gear-mark" aria-hidden="true"><span>"QC"</span></div>
                <div class="affiliate-gear-body">
                    <small>{copy.used_live}</small>
                    <strong>"Neural DSP Quad Cortex"</strong>
                    <p>{copy.product_note}</p>
                    <ExternalLink
                        url=THOMANN_QUAD_CORTEX_URL.to_owned()
                        label=copy.product_cta
                        error=error
                    />
                </div>
            </article>

            <div class="affiliate-gear-support">
                <div>
                    <strong>{copy.general_title}</strong>
                    <p>{copy.general_note}</p>
                </div>
                <ExternalLink
                    url=THOMANN_HOME_URL.to_owned()
                    label=copy.general_cta
                    error=error
                />
            </div>

            <p class="affiliate-disclosure">{copy.disclosure}</p>
        </section>
    }
}
