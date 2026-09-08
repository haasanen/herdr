//! Build identity helpers.

pub const BASE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn channel() -> &'static str {
    non_empty(option_env!("HERDR_BUILD_CHANNEL")).unwrap_or("stable")
}

pub fn build_id() -> Option<&'static str> {
    non_empty(option_env!("HERDR_BUILD_ID"))
}

pub fn metadata() -> Option<&'static str> {
    non_empty(option_env!("HERDR_BUILD_METADATA"))
}

pub fn version() -> String {
    format_version(BASE_VERSION, channel(), build_id(), metadata())
}

fn format_version(
    base_version: &str,
    channel: &str,
    build_id: Option<&str>,
    metadata: Option<&str>,
) -> String {
    let mut version = match channel {
        "stable" => base_version.to_string(),
        channel => match build_id {
            Some(build_id) => format!("{base_version}-{channel}.{build_id}"),
            None => format!("{base_version}-{channel}"),
        },
    };
    if let Some(metadata) = metadata {
        version.push('+');
        version.push_str(metadata);
    }
    version
}

pub fn is_preview() -> bool {
    channel() == "preview"
}

fn non_empty(value: Option<&'static str>) -> Option<&'static str> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn stable_version_defaults_to_cargo_version() {
        assert!(!super::version().is_empty());
    }

    #[test]
    fn stable_version_can_include_fork_metadata() {
        assert_eq!(
            super::format_version("0.9.0", "stable", None, Some("haasanen.1")),
            "0.9.0+haasanen.1"
        );
    }

    #[test]
    fn metadata_follows_preview_identity() {
        assert_eq!(
            super::format_version(
                "0.9.0",
                "preview",
                Some("2026-09-08-abcdef"),
                Some("haasanen.1"),
            ),
            "0.9.0-preview.2026-09-08-abcdef+haasanen.1"
        );
    }
}
