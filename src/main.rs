mod app;
mod config;
mod http;
mod models;
mod services;
mod tui;
mod version;

use anyhow::Result;
use app::App;
use http::HttpClient;

fn main() -> Result<()> {
    let client = HttpClient::new(format!("minit/{}", env!("CARGO_PKG_VERSION")));

    App::new(client, config::default_minecraft_dir()).run()
}
