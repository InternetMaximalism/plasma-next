use actix_web::{web::Data, App, HttpServer};
use log::{error, info};
use zkp::api::{api::api_config, state::ServerState};

lazy_static::lazy_static! {
    static ref SERVER_HOST: String = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    static ref SERVER_PORT: u16 = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string()).parse::<u16>().expect(
        "SERVER_PORT must be a valid port number"
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

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

    let state = ServerState::new();
    let app_data = Data::new(state);
    let host = SERVER_HOST.clone();
    let port = SERVER_PORT.clone();
    info!("Starting server at {host}:{port}");
    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .configure(api_config)
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
