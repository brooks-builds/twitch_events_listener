use eyre::{Context, Result};
use twitch_api::twitch_oauth2::{ClientId, ClientSecret};

pub struct Config {
    pub twitch_client_id: ClientId,
    pub twitch_client_secret: ClientSecret,
    pub twitch_redirect_url: twitch_api::twitch_oauth2::url::Url,
    pub twitch_username: String,
}

impl Config {
    pub fn new_from_env() -> Result<Self> {
        let twitch_client_id = std::env::var("TWITCH_CLIENT_ID")
            .context("missing environment variable TWITCH_CLIENT_ID")?;
        let twitch_client_secret = std::env::var("TWITCH_CLIENT_SECRET")
            .context("missing environment variable TWITCH_CLIENT_SECRET")?;
        let twitch_redirect_url = std::env::var("TWITCH_REDIRECT_URL")
            .context("missing environment variable TWITCH_REDIRECT_URL")?;
        let twitch_username = std::env::var("TWITCH_USERNAME")
            .context("missing environment variable TWITCH_USERNAME")?;

        Ok(Self {
            twitch_client_id: ClientId::new(twitch_client_id),
            twitch_client_secret: ClientSecret::new(twitch_client_secret),
            twitch_redirect_url: twitch_redirect_url
                .parse()
                .context("converting redirect url from environment")?,
            twitch_username,
        })
    }
}
