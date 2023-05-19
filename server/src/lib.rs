mod config;
pub use config::Config;

mod router;

mod handlers;

pub mod error;

use poem::listener::TcpListener;
use poem::middleware::Tracing;
use poem::EndpointExt;
use poem::Server;

use std::env;
use std::io;
use std::sync::Arc;

pub async fn run(mut config: Config) -> io::Result<()> {
    if let Some(level) = config.poem_level.take() {
        env::set_var("RUST_LOG", format!("poem={level}"));
    }
    tracing_subscriber::fmt().init();

    let server = Server::new(TcpListener::bind(("127.0.0.1", config.port))).name("sachima");
    let app = router::new().with(Tracing).data(Arc::new(config));

    server.run(app).await
}
