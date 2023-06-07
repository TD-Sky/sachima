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
mod utils;
use time::format_description::well_known::Rfc3339;
use tracing_subscriber::fmt::time::OffsetTime;
use utils::pswd;

use poem::listener::TcpListener;
use poem::middleware::Tracing;
use poem::EndpointExt;
use poem::Server;
use time::UtcOffset;

use std::env;
use std::io;

pub async fn run(mut config: Config, local_offset: UtcOffset) -> io::Result<()> {
    if let Some(level) = config.poem_log_level.take() {
        env::set_var("RUST_LOG", format!("poem={level}"));
    }

    tracing_subscriber::fmt()
        .with_timer(OffsetTime::new(local_offset, Rfc3339))
        .init();
    db::init(&config.database_url).await;
    pswd::init(&config.password_salt);

    let server = Server::new(TcpListener::bind(("127.0.0.1", config.port))).name("sachima");
    let app = router::new(config).with(Tracing);

    server.run(app).await
}
