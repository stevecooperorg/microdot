use axum::{http::StatusCode, Router};
use std::net::SocketAddr;
use std::path::Path;
use tower_http::services::ServeFile;
use crate::graphviz::ImagePage;
use askama::Template;
use axum::extract::ws::{WebSocket, WebSocketUpgrade, Message};
use axum::routing::get;
use std::sync::{Arc, Mutex};
use futures::{StreamExt, SinkExt};
use tokio::sync::mpsc::UnboundedReceiver;

// Shared state for websocket clients
pub type Clients = Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<Message>>>>;

pub async fn run_web_server(
    port: u16,
    svg_path: impl AsRef<Path>,
    html_path: impl AsRef<Path>,
    json_path: impl AsRef<Path>,
    mut reload_rx: UnboundedReceiver<()>,
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

    let clients: Clients = Arc::new(Mutex::new(Vec::new()));

    let html_handler = get(move || async {
        let html = ImagePage::new("Microdot", "/svg");
        html.render().map(|html_content| {
            ([(axum::http::header::CONTENT_TYPE, "text/html")], html_content)
        })
            .map(|html_content| (StatusCode::OK, html_content))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    });

    let ws_clients = clients.clone();
    let ws_handler = get(move |ws: WebSocketUpgrade| {
        let ws_clients = ws_clients.clone();
        async move {
            ws.on_upgrade(move |socket| handle_socket(socket, ws_clients))
        }
    });

    let app = Router::new()
        .nest_service("/svg", ServeFile::new(svg_path))
        .nest_service("/json", ServeFile::new(json_path))
        .nest_service("/html", html_handler)
        .route("/ws", ws_handler)
        .with_state(clients.clone())
        .fallback(|| async { (StatusCode::NOT_FOUND, "Not Found") });

    // Spawn a task to listen for reload events and broadcast to clients
    let clients_for_reload = clients.clone();
    tokio::spawn(async move {
        while reload_rx.recv().await.is_some() {
            broadcast_reload(&clients_for_reload);
        }
    });

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| e.into())
}

async fn handle_socket(mut socket: WebSocket, clients: Clients) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    // Add this sender to the list
    {
        let mut clients = clients.lock().unwrap();
        clients.push(tx);
    }

    // Forward messages from server to client
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let _ = sender.send(msg).await;
        }
    });

    // Optionally, handle incoming messages from client (not needed for reload)
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(_msg)) = receiver.next().await {
            // Ignore client messages
        }
    });

    let _ = tokio::join!(send_task, recv_task);
}

/// Call this to broadcast a reload message to all websocket clients
pub fn broadcast_reload(clients: &Clients) {
    let clients = clients.lock().unwrap();
    for tx in clients.iter() {
        let _ = tx.send(Message::Text("reload".to_string()));
    }
}
