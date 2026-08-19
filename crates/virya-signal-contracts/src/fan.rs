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
        (!value.is_empty()
            && value.len() <= 128
            && value.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
            }))
        .then(|| value.to_owned())
    }

    pub fn parse(target: &str) -> Self {
        let target = target.trim();
        let (path_and_query, _) = target.split_once('#').unwrap_or((target, ""));
        let (path, query) = path_and_query
            .split_once('?')
            .unwrap_or((path_and_query, ""));
        if let Some(slug) = query
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .find_map(|(key, value)| (key == "event").then_some(value))
            .and_then(Self::event_slug)
        {
            return Self::Event(Some(slug));
        }
        let clean = path.trim_end_matches('/');
        if let Some(slug) = clean
            .split_once("/live/")
            .and_then(|(_, value)| Self::event_slug(value))
        {
            return Self::Event(Some(slug));
        }
        match clean.rsplit('/').next().unwrap_or(clean) {
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
}
