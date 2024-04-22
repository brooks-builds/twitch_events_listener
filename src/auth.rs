use std::collections::HashMap;

use eyre::{Context, OptionExt, Result};
use reqwest::{redirect::Policy, Url};
use twitch_api::twitch_oauth2::{Scope, UserToken, UserTokenBuilder};

use crate::config::Config;

const SCOPES: [Scope; 1] = [Scope::ChannelReadRedemptions];

pub async fn get_user_token(config: &Config) -> Result<UserToken> {
    let client = reqwest::ClientBuilder::new()
        .redirect(Policy::none())
        .build()?;
    let mut token_builder = UserTokenBuilder::new(
        config.twitch_client_id.clone(),
        config.twitch_client_secret.clone(),
        config.twitch_redirect_url.clone(),
    )
    .set_scopes(SCOPES.into())
    .force_verify(true);
    let (auth_url, _) = token_builder.generate_url();
    let twitch_auth_parts = get_response_url(auth_url).context("getting response url parts")?;
    let token = token_builder
        .get_user_token(&client, &twitch_auth_parts.state, &twitch_auth_parts.code)
        .await
        .context("getting user token")?;

    Ok(token)
}

fn get_response_url(auth_url: Url) -> Result<TwitchAuthParts> {
    println!("Authenticate to Twitch by navigating to the following url: \n\n{auth_url}\n\n");

    let response = rpassword::prompt_password("Paste in the entire URL:")
        .context("getting twitch auth response")?;
    let response_url = twitch_api::twitch_oauth2::url::Url::parse(&response)
        .context("parseing response URL into twitch auth URL")?;
    let twitch_auth_parts =
        TwitchAuthParts::try_from(response_url).context("parsing response url")?;

    Ok(twitch_auth_parts)
}

struct TwitchAuthParts {
    pub state: String,
    pub code: String,
}

impl TryFrom<twitch_api::twitch_oauth2::url::Url> for TwitchAuthParts {
    type Error = eyre::Error;

    fn try_from(
        url: twitch_api::twitch_oauth2::url::Url,
    ) -> std::prelude::v1::Result<Self, Self::Error> {
        let params = url.query_pairs().collect::<HashMap<_, _>>();
        let state = params
            .get("state")
            .ok_or_eyre("missing state when handling auth response")?;
        let code = params
            .get("code")
            .ok_or_eyre("missing code when parsing response auth url")?;

        Ok(Self {
            state: state.to_string(),
            code: code.to_string(),
        })
    }
    // fn from(url: twitch_api::twitch_oauth2::url::Url) -> ResultSelf {

    // }
}
