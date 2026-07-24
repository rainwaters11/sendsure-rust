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

#[doc(hidden)]
pub mod test_support {
    use std::io::Error;
    use std::net::TcpStream;

    pub fn handle_client(stream: TcpStream) -> std::io::Result<()> {
        super::router::handle_client(stream)
    }

    pub fn handle_accept_error(error: Error) -> std::io::Result<()> {
        super::handle_accept_error(error)
    }

    pub fn is_ignorable_connection_error(error: &Error) -> bool {
        super::is_ignorable_connection_error(error)
    }
}
