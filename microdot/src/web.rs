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
    svg_path: &Path,
    html_path: &Path,
    json_path: &Path,
    mut reload_rx: UnboundedReceiver<()>,
) -> Result<(), anyhow::Error> {
    let svg_path = svg_path.to_path_buf();
    let json_path = json_path.to_path_buf();
    let html_path = html_path.to_path_buf();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    
    eprintln!("Serving on http://localhost:{}", port);
    eprintln!("  http://localhost:{}/svg serves {}", port, svg_path.display());
    eprintln!("  http://localhost:{}/html serves {}", port, html_path.display());
    eprintln!("  http://localhost:{}/json serves {}", port, json_path.display());
    eprintln!("  http://localhost:{}/ws - WebSocket endpoint for hot reloading", port);

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

async fn handle_socket(socket: WebSocket, clients: Clients) {
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

    // Handle incoming messages from client (currently just ignore them)
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(_msg)) = receiver.next().await {
            // Ignore client messages for now
        }
    });

    let _ = tokio::join!(send_task, recv_task);
}

/// Broadcast a reload message to all websocket clients
pub fn broadcast_reload(clients: &Clients) {
    let clients = clients.lock().unwrap();
    for tx in clients.iter() {
        let _ = tx.send(Message::Text("reload-svg".to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_tungstenite::connect_async;
    use futures::StreamExt;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_websocket_communication() {
        // Create temporary directory for test files
        let temp_dir = tempdir().unwrap();
        let svg_path = temp_dir.path().join("test.svg");
        let html_path = temp_dir.path().join("test.html");
        let json_path = temp_dir.path().join("test.json");

        // Create some test content
        std::fs::write(&svg_path, "<svg>test</svg>").unwrap();
        std::fs::write(&html_path, "<html>test</html>").unwrap();
        std::fs::write(&json_path, r#"{"test": "data"}"#).unwrap();

        // Create reload channel
        let (reload_tx, reload_rx) = tokio::sync::mpsc::unbounded_channel();

        // Start web server on a fixed port for testing
        let port = 8081;
        let server_handle = tokio::spawn(async move {
            run_web_server(port, &svg_path, &html_path, &json_path, reload_rx).await
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Connect WebSocket client
        let ws_url = format!("ws://localhost:{}/ws", port);
        let (ws_stream, _) = connect_async(ws_url).await.expect("Failed to connect");
        let (_, mut read) = ws_stream.split();

        // Wait a bit for connection to stabilize
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Send a reload message through the channel
        let _ = reload_tx.send(());

        // Wait for the message to be received
        let message = tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            read.next()
        ).await.expect("Timeout waiting for message");

        // Verify the message
        if let Some(Ok(message)) = message {
            assert_eq!(message.to_text().unwrap(), "reload-svg");
        } else {
            panic!("Expected message, got: {:?}", message);
        }

        // Clean up
        server_handle.abort();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_static_file_serving() {
        // Create temporary directory for test files
        let temp_dir = tempdir().unwrap();
        let svg_path = temp_dir.path().join("test.svg");
        let html_path = temp_dir.path().join("test.html");
        let json_path = temp_dir.path().join("test.json");

        // Create some test content
        let svg_content = "<svg>test</svg>";
        let json_content = r#"{"test": "data"}"#;
        std::fs::write(&svg_path, svg_content).unwrap();
        std::fs::write(&html_path, "<html>test</html>").unwrap();
        std::fs::write(&json_path, json_content).unwrap();

        // Create reload channel
        let (_, reload_rx) = tokio::sync::mpsc::unbounded_channel();

        // Start web server
        let port = 8082;
        let server_handle = tokio::spawn(async move {
            run_web_server(port, &svg_path, &html_path, &json_path, reload_rx).await
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Test that static files are served correctly
        let svg_response = reqwest::get(&format!("http://localhost:{}/svg", port))
            .await
            .expect("Failed to fetch SVG");
        assert_eq!(svg_response.status(), 200);
        let svg_text = svg_response.text().await.unwrap();
        assert_eq!(svg_text, svg_content);

        let json_response = reqwest::get(&format!("http://localhost:{}/json", port))
            .await
            .expect("Failed to fetch JSON");
        assert_eq!(json_response.status(), 200);
        let json_text = json_response.text().await.unwrap();
        assert_eq!(json_text, json_content);

        // Clean up
        server_handle.abort();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
