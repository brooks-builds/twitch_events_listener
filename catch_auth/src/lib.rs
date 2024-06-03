use std::sync::{Arc, Mutex};

use axum::{extract::Request, routing::get, Extension, Router};
use eyre::{Context, Result};
use reqwest::Url;
use tokio::sync::oneshot::Sender;

pub async fn start_server(code_sender: Sender<Url>) -> Result<()> {
    println!("starting auth server to catch response from twitch");

    let router = Router::new()
        .route("/", get(get_code))
        .layer(Extension(Arc::new(Mutex::new(Some(code_sender)))));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .context("listening on port")?;

    axum::serve(listener, router)
        .await
        .context("serving twitch code catcher")?;

    Ok(())
}

async fn get_code(
    Extension(code_sender): Extension<Arc<Mutex<Option<Sender<Url>>>>>,
    request: Request,
) {
    let uri = request.uri();
    let url = format!("http://localhost:8080{}", uri.to_string());
    if let Ok(url) = reqwest::Url::parse(&url) {
        let sender = code_sender.lock().unwrap().take().unwrap();

        sender.send(url).unwrap();
    } else {
        eprintln!("failed to parse url from twitch");
    };
}
