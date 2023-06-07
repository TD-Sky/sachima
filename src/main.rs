use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;

use clap::Parser;
use sachima::Config;
use time::UtcOffset;

#[derive(Parser)]
struct Cli {
    #[arg(long, short)]
    config: PathBuf,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let mut config = String::new();
    File::open(cli.config)?.read_to_string(&mut config)?;
    let config: Config = toml::from_str(&config).unwrap();

    // `time` cannot get the current local offset
    // in multithreaded context.
    //
    // refer to <https://github.com/time-rs/time/discussions/421>
    let local_offset = UtcOffset::current_local_offset().unwrap();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(sachima::run(config, local_offset))
}
