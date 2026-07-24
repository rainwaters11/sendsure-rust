use sendsure_rust::{run_demo, serve};

fn main() {
    if std::env::args().any(|a| a == "serve") {
        let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let address = format!("0.0.0.0:{port}");
        if let Err(error) = serve(&address) {
            eprintln!("server error: {error}");
            std::process::exit(1);
        }
    } else {
        run_demo();
    }
}
