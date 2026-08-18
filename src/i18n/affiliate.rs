use super::Language;

#[derive(Clone, Copy)]
pub(crate) struct AffiliateGearCopy {
    pub(crate) section_aria: &'static str,
    pub(crate) eyebrow: &'static str,
    pub(crate) title: &'static str,
    pub(crate) intro: &'static str,
    pub(crate) used_live: &'static str,
    pub(crate) product_note: &'static str,
    pub(crate) product_cta: &'static str,
    pub(crate) general_title: &'static str,
    pub(crate) general_note: &'static str,
    pub(crate) general_cta: &'static str,
    pub(crate) disclosure: &'static str,
}

pub(crate) fn gear(language: Language) -> AffiliateGearCopy {
    match language {
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
