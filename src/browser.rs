//! Browser requests are delivered to an attached client, which owns the desktop.

pub(crate) fn pane_browser_env(
    executable: &std::path::Path,
    path: Option<&std::ffi::OsStr>,
) -> Option<(String, std::ffi::OsString)> {
    // A bare executable name works with BROWSER launchers that split whitespace,
    // while PATH preserves spaces in the installation directory without quoting.
    let name = executable.file_name()?.to_str()?;
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._+-".contains(&byte))
    {
        return None;
    }
    let parent = executable.parent()?;
    let paths = std::iter::once(parent.to_path_buf())
        .chain(path.into_iter().flat_map(std::env::split_paths));
    let path = std::env::join_paths(paths).ok()?;
    Some((format!("{name} browser open %s"), path))
}

pub(crate) fn validate_url(url: &str) -> Result<(), &'static str> {
    if url.len() > 8192 || url.chars().any(|ch| ch.is_control() || ch.is_whitespace()) {
        return Err(
            "browser URLs must be at most 8192 bytes and contain no whitespace or controls",
        );
    }
    let address = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .ok_or("only http:// and https:// browser URLs are supported")?;
    if address
        .split(['/', '?', '#'])
        .next()
        .is_none_or(str::is_empty)
    {
        return Err("browser URL must include a host");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_command_handles_spaces_in_install_directory_without_shell_quotes() {
        let original = std::env::join_paths([std::path::PathBuf::from("existing")]).unwrap();
        let executable = std::path::PathBuf::from("install directory").join("herdr");
        let (command, path) = pane_browser_env(&executable, Some(&original)).unwrap();
        assert_eq!(command, "herdr browser open %s");
        assert_eq!(
            std::env::split_paths(&path).collect::<Vec<_>>(),
            vec![
                std::path::PathBuf::from("install directory"),
                std::path::PathBuf::from("existing")
            ]
        );
        assert!(pane_browser_env(std::path::Path::new("herdr with spaces"), None).is_none());
    }

    #[test]
    fn browser_urls_allow_web_links_and_reject_local_commands() {
        for url in [
            "https://example.test/diff/123",
            "http://localhost:3000/?q=a%20b",
        ] {
            assert_eq!(validate_url(url), Ok(()));
        }
        for url in [
            "file:///tmp/a",
            "javascript:alert(1)",
            "https://",
            "https:///x",
            "https://x\nopen",
            "https://x a",
        ] {
            assert!(validate_url(url).is_err(), "accepted {url:?}");
        }
        assert!(validate_url(&format!("https://example.test/{}", "x".repeat(8192))).is_err());
    }
}
