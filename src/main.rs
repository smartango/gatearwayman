use std::env;
use dotenv;

// a service listening http on port 8080
// using axum
use axum::{extract::{Path,State}, routing::get, Router};
use axum::http::Request;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing::Span;
use tracing_subscriber::EnvFilter;

use apimanager_service::static_routes::static_routes;

#[derive(Clone)]
struct Appstate {
    service_manager_service: String,
}

async fn get_services_handler(State(state): State<Appstate>) -> String {
    let uri_part = "/api/services";
    let services = reqwest::get(format!("{}{}", &state.service_manager_service, uri_part))
        .await.unwrap()
        .text()
        .await.unwrap();
    services
}

async fn get_resources_handler(State(state): State<Appstate>) -> String {
    let uri_part = "/api/resources";
    let services = reqwest::get(format!("{}{}", &state.service_manager_service, uri_part))
        .await.unwrap()
        .text()
        .await.unwrap();
    services
}

async fn get_servname_handler(State(state): State<Appstate>, Path(name): Path<String>) -> String {
    let uri_part = "/api/service/";
    let services = reqwest::get(format!("{}{}{}", &state.service_manager_service, uri_part, &name))
        .await.unwrap()
        .text()
        .await.unwrap();
    services
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "apimanager_service=info,tower_http=info".into()),
        )
        .init();

    let _ = dotenv::dotenv();
    let static_pages = static_routes();
    
    let service_manager_service = env::var("SERVICE_MANAGER_SERVICE").expect("SERVICE_MANAGER_SERVICE must be set");
    let state = Appstate {
        service_manager_service,
    };

    let app_routes = Router::new()
    .route("/api/services", get(get_services_handler))
    .route("/api/service/{name}", get(get_servname_handler))
    .route("/api/resources", get(get_resources_handler))
    .with_state(state);
 
    let app = Router::new()
    .merge(static_pages)
    .merge(app_routes)
    .layer(
        TraceLayer::new_for_http()
            .on_request(|request: &Request<_>, _span: &Span| {
                info!(method = %request.method(), uri = %request.uri(), "incoming request");
            }),
    );
    // run it
    let port: u16 = env::var("PORT").unwrap_or_else(|_| "3000".to_string()).parse().unwrap();

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
