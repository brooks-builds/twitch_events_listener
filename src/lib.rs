mod auth;
pub mod config;
pub mod stream_event;
mod websocket_handler;

use ::time::OffsetDateTime;
use auth::get_user_token;
use chrono::{DateTime, TimeZone, Utc};
use config::Config;
use core::time;
use eyre::{Context, OptionExt, Result};
use std::{sync::mpsc::Sender, time::Duration};
use stream_event::StreamEvent;
use twitch_api::{helix::channels::get_ad_schedule, HelixClient};
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

    let get_ad_request = get_ad_schedule::GetAdScheduleRequest::broadcaster_id(streamer.id.clone());
    let response = twitch_helix_client
        .req_get(get_ad_request, &user_token)
        .await
        .context("getting ad schedule")?;
    let next_ad_in = response.data.unwrap().next_ad_at.unwrap_or_default();
    let chrono_time = DateTime::from_timestamp(next_ad_in as i64, 0);
    let time_time = OffsetDateTime::from_unix_timestamp(next_ad_in as i64)
        .context("creating offset date time")?;

    dbg!(chrono_time, time_time);

    websocket
        .run(&twitch_helix_client, &user_token, &streamer)
        .await
        .context("running websocket")?;

    Ok(())
}
