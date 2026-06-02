use axum::{Router, routing::get, serve};
use chrono::Utc;
use log::LevelFilter;
use log::info;
use tokio::net::TcpListener;

mod handlers;
mod history;
mod models;
mod parser;
mod proto;
mod settings;
mod state;

use handlers::*;
use settings::Settings;

#[tokio::main]
async fn main() {
    let settings = Settings::new().expect("Parse settings error");

    let log_level = settings.log.level.parse().unwrap_or(LevelFilter::Warn);
    let _ = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                Utc::now(),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log_level)
        .chain(std::io::stdout())
        .apply();

    let app_state = state::AppState::new(&settings);

    let app = Router::new()
        .route("/bitrix/pub/", get(get_stats).post(push_handler))
        .route("/server-stat/", get(get_stats))
        .route("/bitrix/subws/", get(bitrix_ws_handler))
        .route("/bitrix/sub/", get(bitrix_long_poll_handler))
        .with_state(app_state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", settings.general.port))
        .await
        .unwrap();

    info!(
        "starting HTTP server at http://0.0.0.0:{}",
        settings.general.port
    );
    serve(listener, app).await.unwrap();
}
