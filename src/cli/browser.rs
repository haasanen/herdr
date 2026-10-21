use crate::api::schema::{BrowserOpenParams, Method, Request};

pub(super) fn run_browser_command(args: &[String]) -> std::io::Result<i32> {
    let [action, url] = args else {
        eprintln!("usage: herdr browser open <http-or-https-url>");
        return Ok(2);
    };
    if action != "open" {
        eprintln!("usage: herdr browser open <http-or-https-url>");
        return Ok(2);
    }
    if let Err(error) = crate::browser::validate_url(url) {
        eprintln!("{error}");
        return Ok(2);
    }
    let response = super::send_request(&Request {
        id: "cli:browser:open".into(),
        method: Method::BrowserOpen(BrowserOpenParams { url: url.clone() }),
    })?;
    if response.get("error").is_some() {
        eprintln!("{response}");
        return Ok(1);
    }
    if response["result"]["queued"].as_bool() != Some(true) {
        eprintln!("the server did not queue the browser request");
        return Ok(1);
    }
    Ok(0)
}
