use actix_web::{middleware::Logger, web, App, HttpServer};
use bitrix_channels::{Parser, Signature};
use log::info;
use std::env;
mod app;
mod message;
mod server;
mod session;
mod settings;
mod utils;

use settings::Settings;

#[allow(clippy::derive_partial_eq_without_eq)]
mod items;
/*mod items {
    include!("proto.rs");
    //include!(concat!(env!("OUT_DIR"), "/_.rs"));
}
*/

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = Settings::new().expect("Parse settings error");

    if env::var("RUST_LOG").ok().is_none() {
        env::set_var("RUST_LOG", settings.log.level.clone());
    }

    env_logger::init();

    info!(
        "starting HTTP server at http://0.0.0.0:{}",
        settings.general.port
    );

    info!("log level set to {}", env::var("RUST_LOG").unwrap());

    let parser = match settings.security.enabled {
        true => Parser::new(true, Signature::new(settings.security.key.clone())),
        false => Parser::default(),
    };

    info!("security parser is {}", parser.get_status());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(parser.clone()))
            .configure(app::routes_configure)
            .wrap(Logger::default())
    })
    .workers(settings.general.workers)
    .bind(("0.0.0.0", settings.general.port))?
    .run()
    .await
}
