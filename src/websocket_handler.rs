#![allow(unused_variables)]

use std::{ops::Deref, sync::mpsc::Sender};

use eyre::{bail, Context, Result};
use tokio_tungstenite::{
    connect_async_with_config,
    tungstenite::{self, protocol::WebSocketConfig, Message},
    MaybeTlsStream, WebSocketStream,
};
use twitch_api::{
    eventsub::{self, Event, SessionData},
    helix::users::User,
    twitch_oauth2::UserToken,
    HelixClient, TWITCH_EVENTSUB_WEBSOCKET_URL,
};

use crate::stream_event::StreamEvent;

pub struct WebsocketHandler {
    sender: Sender<StreamEvent>,
}

impl<'a> WebsocketHandler {
    pub fn new(sender: Sender<StreamEvent>) -> Self {
        Self { sender }
    }

    pub async fn run(
        mut self,
        client: &HelixClient<'a, reqwest::Client>,
        token: &UserToken,
        streamer: &User,
    ) -> Result<()> {
        let mut stream = self.connect().await.context("connecting to twitch")?;

        loop {
            tokio::select! (
                Some(message) = futures::StreamExt::next(&mut stream) => {
                    let message = match message {
                        Err(tungstenite::Error::Protocol(tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,)) => {
                            self.connect().await.context("Attempting to reconnect to stream")?;
                            continue;
                        }
                        _ => message.context("extracting message")?,
                    };
                    self.process_message(message, client, token, streamer).await.context("processing message from twitch")?;
                }
            )
        }
    }

    async fn connect(&self) -> Result<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>> {
        let config = WebSocketConfig {
            max_message_size: Some(64 << 20),
            max_frame_size: Some(16 << 20),
            accept_unmasked_frames: false,
            ..WebSocketConfig::default()
        };
        let (socket, _) =
            connect_async_with_config(TWITCH_EVENTSUB_WEBSOCKET_URL.deref(), Some(config), false)
                .await?;

        Ok(socket)
    }

    async fn process_message(
        &mut self,
        message: Message,
        client: &HelixClient<'a, reqwest::Client>,
        token: &UserToken,
        streamer: &User,
    ) -> Result<()> {
        match message {
            Message::Text(message_text) => self
                .handle_message_text(message_text, client, token, streamer)
                .await
                .context("handling message text")?,
            _ => (),
        };

        Ok(())
    }

    async fn handle_message_text(
        &self,
        message: String,
        client: &HelixClient<'a, reqwest::Client>,
        token: &UserToken,
        streamer: &User,
    ) -> Result<()> {
        match twitch_api::eventsub::event::Event::parse_websocket(&message)
            .context("parsing websocket message text")?
        {
            twitch_api::eventsub::EventsubWebsocketData::Welcome {
                metadata: _,
                payload,
            } => self
                .process_welcome_message(payload.session, client, token, streamer)
                .await
                .context("Handling welcome message")?,
            twitch_api::eventsub::EventsubWebsocketData::Keepalive {
                metadata: _,
                payload,
            } => (),
            twitch_api::eventsub::EventsubWebsocketData::Notification { metadata, payload } => {
                self.process_websocket_notification(payload)
                    .context("processing event")?;
            }
            twitch_api::eventsub::EventsubWebsocketData::Revocation { metadata, payload } => {
                bail!("Stream revocated, whatever that means.");
            }
            twitch_api::eventsub::EventsubWebsocketData::Reconnect { metadata, payload } => self
                .process_welcome_message(payload.session, client, token, streamer)
                .await
                .context("reconnecting")?,
            _ => (),
        }
        Ok(())
    }

    async fn process_welcome_message(
        &self,
        session_data: SessionData<'_>,
        client: &HelixClient<'a, reqwest::Client>,
        token: &UserToken,
        streamer: &User,
    ) -> Result<()> {
        let transport = eventsub::Transport::websocket(session_data.id);

        client
            .create_eventsub_subscription(
                eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1::broadcaster_user_id(
                    streamer.id.clone(),
                ),
                transport.clone(),
                token,
            )
            .await
            .context("subscribing to events")?;

        Ok(())
    }

    fn process_websocket_notification(&self, payload: Event) -> Result<()> {
        match payload {
            Event::ChannelPointsCustomRewardRedemptionAddV1(payload) => match payload.message {
                eventsub::Message::Notification(message) => {
                    let title = message.reward.title;
                    let username = message.user_name.to_string();
                    let input = message.user_input;
                    let stream_event = StreamEvent::new(&title.to_lowercase(), username, input);

                    self.sender
                        .send(stream_event)
                        .context("sending redemption message")?;
                }
                _ => todo!(),
            },
            _ => (),
        }

        Ok(())
    }
}
