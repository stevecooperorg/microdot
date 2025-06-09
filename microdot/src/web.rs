use axum::{http::StatusCode, Router};
use std::net::SocketAddr;
use std::path::Path;
use tower_http::services::ServeFile;
use crate::graphviz::ImagePage;
use askama::Template;

pub async fn run_web_server(
    port: u16,
    svg_path: impl AsRef<Path>,
    html_path: impl AsRef<Path>,
    json_path: impl AsRef<Path>,
) -> Result<(), anyhow::Error> {
    let svg_path = svg_path.as_ref();
    let json_path = json_path.as_ref();
    let html_path = html_path.as_ref();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    eprintln!("Serving on http://localhost:{}", port);
    eprintln!(
        "  http://localhost:{}/svg serves {}",
        port,
        svg_path.display()
    );
    eprintln!(
        "  http://localhost:{}/html serves {}",
        port,
        html_path.display()
    );
    eprintln!(
        "  http://localhost:{}/json serves {}",
        port,
        json_path.display()
    );

    // write an axum GET handler for /html that serves the html file
    let html_handler = axum::routing::get(move || async {
        let html = ImagePage::new("Microdot", "/svg");
        html.render().map(|html_content| {
            ([(axum::http::header::CONTENT_TYPE, "text/html")], html_content)
        })
            .map(|html_content| (StatusCode::OK, html_content))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    });
 
    let app = Router::new()
        .nest_service("/svg", ServeFile::new(svg_path))
        .nest_service("/json", ServeFile::new(json_path))
        .nest_service("/html", html_handler)
        .fallback(|| async { (StatusCode::NOT_FOUND, "Not Found") });

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| e.into())
}
