#[test]
fn linux_browser_launcher_passes_the_url_to_herdr_as_one_argument() {
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    let directory = std::env::temp_dir().join(format!(
        "herdr browser launcher {} {}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let helper = directory.join("herdr");
    let recorded = directory.join("args");
    std::fs::write(
        &helper,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$HERDR_BROWSER_TEST_ARGS\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755)).unwrap();
    let original_path = std::env::var_os("PATH").unwrap_or_default();
    std::env::set_var(
        "PATH",
        std::env::join_paths(
            std::iter::once(directory.clone()).chain(std::env::split_paths(&original_path)),
        )
        .unwrap(),
    );
    std::env::set_var("HERDR_BROWSER_TEST_ARGS", &recorded);
    std::env::set_var("BROWSER", "herdr browser open %s");
    let url = "https://example.test/a%20b?q=one&percent=%25s&shell=$()#two";
    webbrowser::open(url).expect("BROWSER must find the Herdr helper");
    let expected = format!("browser\nopen\n{url}\n");
    let deadline = Instant::now() + Duration::from_secs(5);
    let actual = loop {
        if let Ok(text) = std::fs::read_to_string(&recorded) {
            if text == expected {
                break text;
            }
        }
        assert!(Instant::now() < deadline, "browser helper was not invoked");
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(
        actual.lines().collect::<Vec<_>>(),
        vec!["browser", "open", url]
    );
    std::fs::remove_dir_all(directory).unwrap();
}
