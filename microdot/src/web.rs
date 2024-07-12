use axum::{http::StatusCode, Router};
use std::net::SocketAddr;
use std::path::Path;
use tower_http::services::ServeFile;

pub async fn run_web_server(
    port: u16,
    svg_path: impl AsRef<Path>,
    json_path: impl AsRef<Path>,
) -> Result<(), anyhow::Error> {
    let svg_path = svg_path.as_ref();
    let json_path = json_path.as_ref();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    eprintln!("Serving on http://{}", addr);
    eprintln!("  http://{}/svg serves {}", addr, svg_path.display());
    eprintln!("  http://{}/json serves {}", addr, json_path.display());

    let app = Router::new()
        .nest_service("/svg", ServeFile::new(svg_path))
        .nest_service("/json", ServeFile::new(json_path))
        .fallback(|| async { (StatusCode::NOT_FOUND, "Not Found") });

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| e.into())
}
