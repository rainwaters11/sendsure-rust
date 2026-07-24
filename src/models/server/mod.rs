mod cors;
mod router;

use std::io::Error;
use std::net::TcpListener;

use router::handle_client;

pub fn serve(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("SendSure server listening on http://{addr}");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle_client(stream) {
                    if is_ignorable_connection_error(&error) {
                        eprintln!("client connection error (ignored): {error}");
                    } else {
                        eprintln!("client connection error: {error}");
                    }
                }
            }
            Err(error) => {
                handle_accept_error(error)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn handle_accept_error(error: Error) -> std::io::Result<()> {
    if is_ignorable_connection_error(&error) {
        eprintln!("connection accept error (ignored): {error}");
        Ok(())
    } else {
        eprintln!("connection accept error: {error}");
        Err(error)
    }
}

pub(crate) fn is_ignorable_connection_error(error: &Error) -> bool {
    use std::io::ErrorKind;
    matches!(
        error.kind(),
        ErrorKind::BrokenPipe
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::UnexpectedEof
    )
}

#[cfg(test)]
mod tests {
    use super::{handle_accept_error, handle_client, is_ignorable_connection_error};
    use std::io::{Error, ErrorKind, Read, Write};
    use std::net::{Shutdown, TcpListener, TcpStream};
    use std::thread;

    fn round_trip(request: &str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral listener");
        let addr = listener.local_addr().expect("read listener address");
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept connection");
            handle_client(stream).expect("serve one client");
        });

        let mut client = TcpStream::connect(addr).expect("connect to test server");
        client.write_all(request.as_bytes()).expect("send request");
        client
            .shutdown(Shutdown::Write)
            .expect("shutdown write side");
        let mut response = String::new();
        client.read_to_string(&mut response).expect("read response");
        server.join().expect("join server thread");
        response
    }

    #[test]
    fn options_evaluate_returns_success_with_required_cors_headers() {
        let response = round_trip(
            "OPTIONS /api/evaluate HTTP/1.1\r\nHost: example\r\nContent-Length: 0\r\n\r\n",
        );
        assert!(response.starts_with("HTTP/1.1 204 No Content\r\n"));
        assert!(response.contains("Access-Control-Allow-Origin: *\r\n"));
        assert!(response.contains("Access-Control-Allow-Methods: POST, OPTIONS\r\n"));
        assert!(response.contains("Access-Control-Allow-Headers: Content-Type\r\n"));
        assert!(response.contains("Content-Length: 0\r\n"));
    }

    #[test]
    fn ignorable_connection_kinds_remain_ignorable() {
        for kind in [
            ErrorKind::BrokenPipe,
            ErrorKind::ConnectionReset,
            ErrorKind::ConnectionAborted,
            ErrorKind::UnexpectedEof,
        ] {
            let error = Error::new(kind, "ignorable");
            assert!(
                is_ignorable_connection_error(&error),
                "expected {:?} to be ignorable",
                kind
            );
            assert!(
                handle_accept_error(error).is_ok(),
                "ignorable listener errors should be logged and ignored"
            );
        }
    }

    #[test]
    fn non_ignorable_accept_error_is_returned() {
        let error = Error::new(ErrorKind::AddrInUse, "non-ignorable");
        let returned = handle_accept_error(error).expect_err("non-ignorable errors should return");
        assert_eq!(returned.kind(), ErrorKind::AddrInUse);
    }

    #[test]
    fn favicon_routes_serve_shield_mark_svg() {
        for path in ["/favicon.svg", "/assets/sendsure-mark.svg"] {
            let request = format!("GET {path} HTTP/1.1\r\nHost: example\r\n\r\n");
            let response = round_trip(&request);
            assert!(
                response.starts_with("HTTP/1.1 200 OK\r\n"),
                "expected 200 for {path}"
            );
            assert!(
                response.contains("Content-Type: image/svg+xml\r\n"),
                "expected svg content type for {path}"
            );
            assert!(
                response.contains("<svg xmlns=\"http://www.w3.org/2000/svg\""),
                "expected svg body for {path}"
            );
        }
    }
}
