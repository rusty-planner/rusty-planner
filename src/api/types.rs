use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateGroupRequest {
    pub guild_id: String,
    pub group_name: String,
    pub group_description: Option<String>,
    pub group_image: Option<String>,
}

#[derive(Serialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
}

impl From<crate::api::database::Session> for User {
    fn from(session: crate::api::database::Session) -> Self {
        User {
            id: session.discord_id,
            name: session.discord_name,
            avatar: session.discord_avatar,
        }
    }
}
impl From<crate::api::discord::CurrentUserResponse> for User {
    fn from(user: crate::api::discord::CurrentUserResponse) -> Self {
        User {
            id: user.id,
            name: user.username,
            avatar: user.avatar,
        }
    }
}
