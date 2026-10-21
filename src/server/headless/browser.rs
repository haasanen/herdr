use super::*;

impl HeadlessServer {
    pub(super) fn handle_browser_open_api(
        &mut self,
        request_id: &str,
        request: &api::schema::BrowserOpenParams,
    ) -> String {
        let error = |code: &str, message: &str| {
            serde_json::json!({
                "id": request_id,
                "error": { "code": code, "message": message },
            })
            .to_string()
        };
        if let Err(message) = crate::browser::validate_url(&request.url) {
            return error("invalid_url", message);
        }
        let eligible = |client: &ClientConnection| {
            matches!(client.mode, ClientConnectionMode::ClientShell)
                && client.browser_open
                && client.writer.is_some()
        };
        let client_id = self
            .foreground_client_id
            .filter(|id| self.clients.get(id).is_some_and(&eligible))
            .or_else(|| {
                self.clients
                    .iter()
                    .filter(|(_, client)| eligible(client))
                    .max_by_key(|(id, client)| (client.last_activity, **id))
                    .map(|(id, _)| *id)
            });
        let Some(client_id) = client_id else {
            return error(
                "no_browser_client",
                "no attached browser-capable client is available",
            );
        };
        let data = match serde_json::to_string(request) {
            Ok(data) => data,
            Err(_) => return error("serialization_error", "could not encode browser request"),
        };
        if !self.send_to_client(
            client_id,
            ServerMessage::EndpointControl {
                kind: crate::protocol::endpoint::BROWSER_OPEN_KIND.into(),
                data,
            },
        ) {
            return error(
                "browser_client_disconnected",
                "the browser client disconnected before delivery",
            );
        }
        serde_json::json!({ "id": request_id, "result": { "type": "browser_open", "queued": true } }).to_string()
    }
}
