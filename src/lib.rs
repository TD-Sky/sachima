mod config;
pub use config::Config;

mod error;
mod handlers;
mod models;
mod reply;
mod router;
pub mod utils;

use poem::listener::TcpListener;
use poem::middleware::Tracing;
use poem::EndpointExt;
use poem::Server;

use std::env;
use std::io;
use std::sync::Arc;

pub async fn run(config: Config) -> io::Result<()> {
    let Config {
        port,
        poem_level,
        workspace,
        max_upload,
    } = config;

    if let Some(level) = poem_level {
        env::set_var("RUST_LOG", format!("poem={level}"));
    }
    tracing_subscriber::fmt().init();

    let server = Server::new(TcpListener::bind(("127.0.0.1", port))).name("sachima");
    let app = router::new()
        .with(Tracing)
        .data(Arc::new(workspace))
        .data(max_upload);

    server.run(app).await
}
