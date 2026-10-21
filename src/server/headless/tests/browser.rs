use super::*;

fn add_browser_client(
    server: &mut HeadlessServer,
    id: u64,
    supported: bool,
    active: bool,
) -> std::sync::mpsc::Receiver<Vec<u8>> {
    let (writer, control, _) = test_client_writer();
    let mut client = ClientConnection::new(
        (80, 24),
        crate::kitty_graphics::HostCellSize::default(),
        id,
        RenderEncoding::SemanticFrame,
        Some(writer),
    );
    client.browser_open = supported;
    client.shell_surface_active = active;
    server.clients.insert(id, client);
    control
}

fn browser_request(server: &mut HeadlessServer, url: &str) -> serde_json::Value {
    serde_json::from_str(&server.handle_browser_open_api(
        "open-test",
        &api::schema::BrowserOpenParams { url: url.into() },
    ))
    .unwrap()
}

#[test]
fn browser_request_uses_one_capable_client_without_changing_workspaces() {
    let mut server = test_headless_server();
    let foreground = add_browser_client(&mut server, 1, true, true);
    let background = add_browser_client(&mut server, 2, true, false);
    server.foreground_client_id = Some(1);
    let workspace_count = server.app.state.workspaces.len();
    let url = "https://example.test/diff/123?foo=bar&x=1#code";
    assert_eq!(browser_request(&mut server, url)["result"]["queued"], true);
    let ServerMessage::EndpointControl { kind, data } =
        read_server_message(foreground.recv_timeout(Duration::from_secs(1)).unwrap())
    else {
        panic!("expected optional browser request");
    };
    assert_eq!(kind, crate::protocol::endpoint::BROWSER_OPEN_KIND);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&data).unwrap()["url"],
        url
    );
    assert!(background.try_recv().is_err());
    assert!(foreground.try_recv().is_err());
    assert_eq!(server.app.state.workspaces.len(), workspace_count);
}

#[test]
fn browser_request_can_use_a_background_client_when_foreground_is_old() {
    let mut server = test_headless_server();
    let old = add_browser_client(&mut server, 1, false, true);
    let capable = add_browser_client(&mut server, 2, true, false);
    server.foreground_client_id = Some(1);
    assert_eq!(
        browser_request(&mut server, "https://example.test")["result"]["queued"],
        true
    );
    assert!(capable.recv_timeout(Duration::from_secs(1)).is_ok());
    assert!(old.try_recv().is_err());
}

#[test]
fn browser_request_rejects_unsafe_urls_and_missing_browser_clients() {
    let mut server = test_headless_server();
    assert_eq!(
        browser_request(&mut server, "https://example.test")["error"]["code"],
        "no_browser_client"
    );
    let capable = add_browser_client(&mut server, 1, true, true);
    assert_eq!(
        browser_request(&mut server, "file:///etc/passwd")["error"]["code"],
        "invalid_url"
    );
    assert!(capable.try_recv().is_err());
    server.clients.get_mut(&1).unwrap().writer = None;
    assert_eq!(
        browser_request(&mut server, "https://example.test")["error"]["code"],
        "no_browser_client"
    );
}

#[test]
fn browser_request_does_not_report_delivery_to_a_closed_writer() {
    let mut server = test_headless_server();
    let control = add_browser_client(&mut server, 1, true, true);
    drop(control);
    assert_eq!(
        browser_request(&mut server, "https://example.test")["error"]["code"],
        "browser_client_disconnected"
    );
}
