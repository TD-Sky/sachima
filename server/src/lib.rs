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

pub async fn run(mut config: Config) -> io::Result<()> {
    if let Some(level) = config.poem_level.take() {
        env::set_var("RUST_LOG", format!("poem={level}"));
    }
    tracing_subscriber::fmt().init();

    let app = router::new().with(Tracing);
    let server = Server::new(TcpListener::bind(("127.0.0.1", config.port))).name("sachima");

    server.run(app).await
}
