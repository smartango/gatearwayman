use axum::{
    http::{header, HeaderMap, HeaderValue, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    Router,
};
use crate::assets::{STATIC_FILEMAP, STATIC_FILEMAP_MIME};

fn replace_base_href(html: &str, new_base: &str) -> String {
    let re = regex::Regex::new(r#"<base href="[^"]*".*/>"#).unwrap();
    re.replace_all(html, format!(r#"<base href="{}"/>"#, new_base)).to_string()
}

pub fn static_routes() -> Router {
    let prefix = "/";
    let mut static_pages = Router::new();
    let base_href = std::env::var("BASE_HREF").unwrap_or_else(|_| "/".to_string());
    let mut index_html = String::new();

    for (k, v) in STATIC_FILEMAP.entries() {
        let mime = STATIC_FILEMAP_MIME.get(k).unwrap_or(&"application/octet-stream");
        let k = format!("{}{}", prefix, k);
        if k == format!("{}index.html", prefix) {
            let k = format!("{}", prefix);
            let v2 = replace_base_href(v, &base_href);
            println!("BASE_HREF: {}, replaced index.html with {}", base_href, v2);
            index_html = v2.clone();
            static_pages = static_pages.clone().route(&k,
                get(move || async move {
                let mut headers = HeaderMap::new();
                headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
                (headers, v2.clone()).into_response()
            }));
        }
        static_pages = static_pages.clone().route(&k,
            get(move || async move {
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(*mime));
            (headers, *v).into_response()
        }));
    }

    static_pages = static_pages.fallback(move |uri: Uri| {
        let index_html = index_html.clone();
        async move {
            if uri.path().starts_with("/api/") || uri.path() == "/api" {
                let mut headers = HeaderMap::new();
                headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"));
                return (StatusCode::NOT_FOUND, headers, "Not Found").into_response();
            }

            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
            (StatusCode::OK, headers, index_html).into_response()
        }
    });

    static_pages
} 