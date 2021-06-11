use serde::Deserialize;

use crate::api::AppData;

pub static API_ENDPOINT: &str = "https://discord.com/api/v8";
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
    pub code: u64,
}

impl std::error::Error for ErrorResponse {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Discord API Error {} \"{}\"", self.code, self.message)
    }
}

#[derive(Debug, Deserialize)]
pub struct OauthErrorResponse {
    pub error: Option<String>,
    pub error_description: Option<String>,
    pub message: Option<String>,
    pub code: Option<u64>,
}

impl std::error::Error for OauthErrorResponse {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for OauthErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let (Some(error), Some(error_description)) = (&self.error, &self.error_description) {
            write!(
                f,
                "Discord API (oauth) Error {} \"{}\"",
                error, error_description
            )
        } else if let (Some(code), Some(message)) = (&self.code, &self.message) {
            write!(f, "Discord API Error {} \"{}\"", code, message)
        } else {
            write!(f, "Discord API Error {:?}", self)
        }
    }
}

#[derive(Debug, serde::Serialize, Deserialize)]
pub struct OauthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
    pub refresh_token: String,
    pub scope: String,
}

async fn req<
    R: serde::de::DeserializeOwned,
    E: 'static + std::error::Error + serde::de::DeserializeOwned,
>(
    req: reqwest::RequestBuilder,
) -> Result<R> {
    let text = req.send().await?.text().await?;
    let res = serde_json::from_str::<R>(&text);
    match res {
        Ok(r) => Ok(r),
        Err(_) => {
            let err = serde_json::from_str::<E>(&text)?;
            Err(Box::new(err))
        }
    }
}

pub async fn oauth_token(
    data: &AppData,
    code: &str,
    redirect_uri: &str,
) -> Result<OauthTokenResponse> {
    req::<OauthTokenResponse, OauthErrorResponse>(
        data.http_client
            .post(&format!("{}/oauth2/token", API_ENDPOINT))
            .form(&[
                ("client_id", data.client_id),
                ("client_secret", data.client_secret),
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", redirect_uri),
            ]),
    )
    .await
}

#[derive(Debug, Deserialize)]
pub struct CurrentUserResponse {
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub discriminator: String,
    pub public_flags: u64,
    pub flags: u64,
}

pub async fn current_user(data: &AppData, token: &str) -> Result<CurrentUserResponse> {
    req::<CurrentUserResponse, ErrorResponse>(
        data.http_client
            .get(&format!("{}/users/@me", API_ENDPOINT))
            .header("Authorization", format!("Bearer {}", token)),
    )
    .await
}

#[derive(Debug, serde::Serialize, Deserialize)]
pub struct CurrentGuildsItem {
    pub id: String,
    pub name: String,
    // pub icon: Option<String>,
    pub owner: bool,
    // pub permissions: String,
    // pub features: Vec<String>,
}
pub type CurrentGuildsResponse = Vec<CurrentGuildsItem>;

pub async fn current_user_guilds(data: &AppData, token: &str) -> Result<CurrentGuildsResponse> {
    req::<CurrentGuildsResponse, ErrorResponse>(
        data.http_client
            .get(&format!("{}/users/@me/guilds", API_ENDPOINT))
            .header("Authorization", format!("Bearer {}", token)),
    )
    .await
}
