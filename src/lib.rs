mod config;
pub use config::Config;

mod db;
mod entity;
mod error;
mod handlers;
mod middlewares;
mod models;
mod reply;
mod router;
pub mod utils;

use poem::listener::TcpListener;
use poem::middleware::Tracing;
use poem::EndpointExt;
use poem::Server;
use utils::pswd;

use std::env;
use std::io;

pub async fn run(mut config: Config) -> io::Result<()> {
    if let Some(level) = config.poem_log_level.take() {
        env::set_var("RUST_LOG", format!("poem={level}"));
    }
    tracing_subscriber::fmt().init();

    db::init(&config.database_url).await;
    pswd::init(&config.password_salt);

    let server = Server::new(TcpListener::bind(("127.0.0.1", config.port))).name("sachima");
    let app = router::new(config).with(Tracing);

    server.run(app).await
}
