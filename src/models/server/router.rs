use std::io::Write;
use std::net::TcpStream;

use crate::models::frontend::{APP_JS, INDEX_HTML, LOGO_HORIZONTAL_SVG, MARK_SVG, STYLES_CSS};
use crate::{demo_scenarios, evaluate, parse_http_request, Intent, Registries};

use super::cors::{ACCESS_CONTROL_ALLOW_ORIGIN, EVALUATE_OPTIONS_EXTRA_HEADERS};

pub(crate) fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let (request, body) = parse_http_request(&mut stream)?;
    let first = request.lines().next().unwrap_or_default();
    let (status, content_type, response, extra_headers) = route_request(first, &body);
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\n{ACCESS_CONTROL_ALLOW_ORIGIN}{extra_headers}Content-Length: {}\r\nConnection: close\r\n\r\n{response}",
        response.len()
    )
}

fn route_request(first: &str, body: &str) -> (&'static str, &'static str, String, &'static str) {
    if first.starts_with("OPTIONS /api/evaluate ") {
        (
            "204 No Content",
            "text/plain; charset=utf-8",
            String::new(),
            EVALUATE_OPTIONS_EXTRA_HEADERS,
        )
    } else if first.starts_with("GET /health ") {
        (
            "200 OK",
            "application/json",
            serde_json::json!({"status":"ok"}).to_string(),
            "",
        )
    } else if first.starts_with("GET /api/scenarios ") {
        (
            "200 OK",
            "application/json",
            serde_json::to_string(&demo_scenarios()).unwrap_or_else(|_| "[]".to_string()),
            "",
        )
    } else if first.starts_with("POST /api/evaluate ") {
        match serde_json::from_str::<Intent>(body) {
            Ok(intent) => (
                "200 OK",
                "application/json",
                serde_json::to_string(&evaluate(&intent, &Registries::default()))
                    .unwrap_or_else(|_| "{}".to_string()),
                "",
            ),
            Err(error) => (
                "400 Bad Request",
                "application/json",
                serde_json::json!({"error": error.to_string()}).to_string(),
                "",
            ),
        }
    } else if first.starts_with("GET / ") {
        (
            "200 OK",
            "text/html; charset=utf-8",
            INDEX_HTML.to_string(),
            "",
        )
    } else if first.starts_with("GET /app.js ") {
        ("200 OK", "application/javascript", APP_JS.to_string(), "")
    } else if first.starts_with("GET /styles.css ") {
        ("200 OK", "text/css", STYLES_CSS.to_string(), "")
    } else if first.starts_with("GET /favicon.svg ")
        || first.starts_with("GET /assets/sendsure-mark.svg ")
    {
        (
            "200 OK",
            "image/svg+xml",
            MARK_SVG.to_string(),
            "Cache-Control: public, max-age=86400\r\n",
        )
    } else if first.starts_with("GET /assets/sendsure-logo-horizontal.svg ") {
        (
            "200 OK",
            "image/svg+xml",
            LOGO_HORIZONTAL_SVG.to_string(),
            "",
        )
    } else {
        (
            "404 Not Found",
            "application/json",
            serde_json::json!({"error":"not found"}).to_string(),
            "",
        )
    }
}
