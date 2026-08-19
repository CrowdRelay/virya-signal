include!("fan_wire.generated.rs");

/// Typed navigation target shared by push/App Links and Fan Home actions.
/// Parsing exact query keys avoids bugs such as `not_event=` matching `event=`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FanTarget {
    Signal,
    Event(Option<String>),
    Wallet,
    Merch,
    Area,
    Profile,
}

impl FanTarget {
    fn event_slug(value: &str) -> Option<String> {
        let value = value.trim();
        if value.is_empty() || value.len() > 128 {
            return None;
        }
        for byte in value.bytes() {
            if !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_'))
            {
                return None;
            }
        }
        Some(value.to_owned())
    }

    pub fn parse(target: &str) -> Self {
        let target = target.trim();
        let path_and_query = target.find('#').map_or(target, |index| &target[..index]);
        let (path, query) = match path_and_query.find('?') {
            Some(index) => (&path_and_query[..index], &path_and_query[index + 1..]),
            None => (path_and_query, ""),
        };

        for pair in query.split('&') {
            if let Some(value) = pair.strip_prefix("event=") {
                if let Some(slug) = Self::event_slug(value) {
                    return Self::Event(Some(slug));
                }
            }
        }

        let clean = path.trim_end_matches('/');
        if let Some(index) = clean.find("/live/") {
            if let Some(slug) = Self::event_slug(&clean[index + 6..]) {
                return Self::Event(Some(slug));
            }
        }

        let leaf = clean.rfind('/').map_or(clean, |index| &clean[index + 1..]);
        match leaf {
            "area" => Self::Area,
            "merch" => Self::Merch,
            "tickets" | "wallet" => Self::Wallet,
            "profile" => Self::Profile,
            "events" => Self::Event(None),
            _ => Self::Signal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FanTarget;

    #[test]
    fn exact_event_query_key_only() {
        assert_eq!(FanTarget::parse("/signal?not_event=x"), FanTarget::Signal);
        assert_eq!(
            FanTarget::parse("/signal?event=wro-1"),
            FanTarget::Event(Some("wro-1".into()))
        );
        assert_eq!(
            FanTarget::parse("/signal?event=https://evil.invalid"),
            FanTarget::Signal
        );
        assert_eq!(FanTarget::parse("/live/wro-1/extra"), FanTarget::Signal);
        assert_eq!(
            FanTarget::parse(&format!("/signal?event={}", "a".repeat(129))),
            FanTarget::Signal
        );
    }

    #[test]
    fn route_mapping_and_fragments_stay_stable() {
        assert_eq!(FanTarget::parse("/area#map"), FanTarget::Area);
        assert_eq!(FanTarget::parse("/tickets/"), FanTarget::Wallet);
        assert_eq!(FanTarget::parse("/events"), FanTarget::Event(None));
        assert_eq!(
            FanTarget::parse("https://virya.music/live/wro-1"),
            FanTarget::Event(Some("wro-1".into()))
        );
    }
}
