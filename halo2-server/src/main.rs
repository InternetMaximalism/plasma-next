use actix::Actor;
use actix_web::{web::Data, App, HttpServer};
use api::api_config;
use dotenv::dotenv;
use log::{error, info};
use state::{SnarkState, StateActor};

pub mod api;
pub mod snark_processor;
pub mod state;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let server_host: String =
        std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let server_port: u16 = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid port number");
    let log_level: String = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

    env_logger::init_from_env(env_logger::Env::new().default_filter_or(log_level.as_str()));

    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(location) = panic_info.location() {
            error!(
                "Panic occurred in file '{}' at line {}",
                location.file(),
                location.line()
            );
        } else {
            error!("Panic occurred but can't get location information...");
        }
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            error!("Panic payload: {:?}", s);
        }
    }));

    let state = SnarkState::new();
    let app_data = Data::new(state);
    let addr = StateActor::new().start();
    info!("Starting server at {server_host}:{server_port}");

    #[cfg(feature = "debug")]
    log::warn!("Debug Mode");

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .app_data(Data::new(addr.clone()))
            .configure(api_config)
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind((server_host.as_str(), server_port))?
    .run()
    .await
}
