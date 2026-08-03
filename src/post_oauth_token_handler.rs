use axum::{extract::State, http::Request, response::IntoResponse};
use axum::http::StatusCode;
use crate::Appstate;

pub fn post_oauth_token_handler(
    State(state): State<Appstate>,
    req: Request<String>,
 ) -> impl IntoResponse {
    let uri_part = "/api/oauth/token";
    let client = reqwest::Client::new();
    let mut request_builder = client.post(format!("{}{}", &state.service_manager_service, uri_part));
    
    // copy headers from the incoming request to the outgoing request
    for (key, value) in req.headers().iter() {
        request_builder = request_builder.header(key, value);
    }

    // set the body of the outgoing request to be the same as the incoming request
    let body = req.into_body();
    request_builder = request_builder.body(body);

    // send the request and get the response
    let response = match request_builder.send() {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("Error sending request: {}", err);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response();
        }
    };

    // get the status code and body of the response
    let status = response.status();
    let body = match response.text() {
        Ok(text) => text,
        Err(err) => {
            eprintln!("Error reading response body: {}", err);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response();
        }
    };

    // return the status code and body of the response
    (status, body).into_response()
}