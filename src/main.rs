use std::env;
use dotenv;

// a service listening http on port 8080
// using axum
use axum::{extract::{Path,State}, routing::get, routing::post, Json, Router};
use axum::http::Request;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use jsonwebtoken::{decode, Algorithm, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing::Span;
use tracing_subscriber::EnvFilter;

use apimanager_service::static_routes::static_routes;

#[derive(Clone)]
struct Appstate {
    service_manager_service: String,
    id_provider_type: String,
    rsa_key_priv: Option<String>,
    rsa_key_pub: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct Claims {
    uid: u64,
    perms: Vec<String>,
}

#[derive(Serialize)]
struct TokenResponse {
    token: String,
}

#[derive(Serialize)]
struct VoidJson {}

#[derive(Deserialize)]
struct Credentials {
    login: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginBody {
    credentials: Credentials,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn get_services_handler(
    State(state): State<Appstate>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let bearer_token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    let authorized = match (bearer_token, state.rsa_key_pub.as_deref()) {
        (Some(token), Some(public_key_pem)) => {
            let decoding_key = match DecodingKey::from_rsa_pem(public_key_pem.as_bytes()) {
                Ok(key) => key,
                Err(_) => {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(ErrorResponse {
                            error: "unauthorized".to_string(),
                        }),
                    )
                        .into_response();
                }
            };

            let validation = Validation::new(Algorithm::RS256);
            decode::<Claims>(token, &decoding_key, &validation).is_ok()
        }
        _ => false,
    };

    if !authorized {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "unauthorized".to_string(),
            }),
        )
            .into_response();
    }

    let uri_part = "/api/services";
    let client = reqwest::Client::new();
    let mut request_builder = client.get(format!("{}{}", &state.service_manager_service, uri_part));

    for (key, value) in headers.iter() {
        request_builder = request_builder.header(key, value);
    }

    let services = request_builder
        .send()
        .await.unwrap()
        .text()
        .await.unwrap();
    services.into_response()
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

async fn post_login_token_handler(
    State(state): State<Appstate>,
    payload: Result<Json<LoginBody>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    if state.id_provider_type != "internal" {
        return (StatusCode::OK, Json(VoidJson {})).into_response();
    }
    let body = match payload {
        Ok(Json(body)) => body,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "unauthorized".to_string(),
                }),
            )
                .into_response();
        }
    };

    if body.credentials.login != "test" || body.credentials.password != "test" {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "unauthorized".to_string(),
            }),
        )
            .into_response();
    }
    // check if post request has a body with credentials: { login, password}
    

    let _public_key_pem = state.rsa_key_pub.as_deref();

    let claims = Claims {
        uid: 1,
        perms: vec!["admin".to_string()],
    };

    let private_key_pem = match &state.rsa_key_priv {
        Some(key) => key,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "missing private key for internal provider".to_string(),
            )
                .into_response();
        }
    };

    let encoding_key = match EncodingKey::from_rsa_pem(private_key_pem.as_bytes()) {
        Ok(key) => key,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("invalid rsa private key: {err}"),
            )
                .into_response();
        }
    };

    let mut header = jsonwebtoken::Header::new(Algorithm::RS256);
    header.typ = Some("JWT".to_string());

    match jsonwebtoken::encode(&header, &claims, &encoding_key) {
        Ok(token) => (StatusCode::OK, Json(TokenResponse { token })).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to sign jwt: {err}"),
        )
            .into_response(),
    }
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
    let id_provider_type = env::var("ID_PROVIDER_TYPE").unwrap_or_else(|_| "internal".to_string());
    let (rsa_key_priv, rsa_key_pub) = if id_provider_type == "internal" {
        info!("Using internal ID provider");
        let (rsa_key_priv, rsa_key_pub) = apimanager_service::gen_rsa_keys::generate_rsa_keypair_2048()
            .expect("failed to generate rsa key pair");
        (Some(rsa_key_priv), Some(rsa_key_pub))
    } else {
        info!("Using external ID provider: {}", id_provider_type);
        (None, None)
    };
    let state = Appstate {
        service_manager_service,
        id_provider_type,
        rsa_key_priv,
        rsa_key_pub,
    };

    let app_routes = Router::new()
    .route("/api/services", get(get_services_handler))
    .route("/api/service/{name}", get(get_servname_handler))
    .route("/api/resources", get(get_resources_handler))
    //.route("/api/oauth/token", post(post_oauth_token_handler))
    .route("/api/login", post(post_login_token_handler))
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
