mod auth;
pub mod config;
pub mod stream_event;
mod websocket_handler;

use auth::get_user_token;
use config::Config;
use eyre::{Context, OptionExt, Result};
use std::sync::mpsc::Sender;
use stream_event::StreamEvent;
use twitch_api::HelixClient;
use websocket_handler::WebsocketHandler;

pub async fn run(config: Config, sender: Sender<StreamEvent>) -> Result<()> {
    println!("Running Twitch Events Listener");

    let user_token = get_user_token(&config)
        .await
        .context("authenticating with twitch")?;
    let twitch_helix_client: HelixClient<reqwest::Client> = HelixClient::default();
    let streamer = twitch_helix_client
        .get_user_from_login(&config.twitch_username, &user_token)
        .await
        .context("getting streamer info")?
        .ok_or_eyre("extracting streamer info")?;
    let websocket = WebsocketHandler::new(sender);

    websocket
        .run(&twitch_helix_client, &user_token, &streamer)
        .await
        .context("running websocket")?;

    Ok(())
}
