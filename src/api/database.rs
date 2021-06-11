use postgres::{types::Type, Client};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

use crate::api::discord::OauthTokenResponse;
use crate::api::error::{Error, ErrorKind};
use crate::api::response::SimpleResponse;
use crate::api::types::User;

pub fn create_session(
    client: &mut Client,
    token: &OauthTokenResponse,
    user: &User,
) -> Result<Uuid, Error> {
    let uuid = Uuid::new_v4();
    let r = client.execute(
        "INSERT INTO sessions (session_token, session_expires, session_refresh_token, session_cookie, discord_id, discord_name, discord_avatar) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    &[&token.access_token, &token.expires_in, &token.refresh_token, &uuid, &user.id, &user.name, &user.avatar]
    ).map_err(|e| Error::map_from(Box::new(e)))?;

    if r == 1 {
        Ok(uuid)
    } else {
        Err(Error {
            kind: ErrorKind::Database(String::from("No rows were updated")),
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Session {
    pub id: i64,
    pub cookie: Uuid,
    pub token: String,
    pub refresh_token: String,
    pub expires: u32,
    pub updated: SystemTime,
    pub discord_id: String,
    pub discord_name: String,
    pub discord_avatar: Option<String>,
}

pub fn get_session(client: &mut Client, session: &Uuid) -> Result<Session, Error> {
    match client.query_one("SELECT session_id, session_token, session_expires, session_refresh_token, session_cookie, session_updated, discord_id, discord_name, discord_avatar FROM sessions WHERE session_cookie=$1", &[&session]) {
        Ok(row) => {
            if row.is_empty() {
                Err(Error::from_simple(SimpleResponse {
                    code: 403,
                    message: String::from("Forbidden"),
                }))
            } else {
                println!("{:?}", row);
                Ok(Session {
                    id: row.try_get(0).map_err(|e| Error::map_from(Box::new(e)))?,
                    token: row.try_get(1).map_err(|e| Error::map_from(Box::new(e)))?,
                    expires: row.try_get(2).map_err(|e| Error::map_from(Box::new(e)))?,
                    refresh_token: row.try_get(3).map_err(|e| Error::map_from(Box::new(e)))?,
                    cookie: row.try_get(4).map_err(|e| Error::map_from(Box::new(e)))?,
                    updated: row.try_get(5).map_err(|e| Error::map_from(Box::new(e)))?,
                    discord_id: row.try_get(6).map_err(|e| Error::map_from(Box::new(e)))?,
                    discord_name: row.try_get(7).map_err(|e| Error::map_from(Box::new(e)))?,
                    discord_avatar: row.try_get(8).map_err(|e| Error::map_from(Box::new(e)))?,
                })
            }
        },
        Err(e) => Err(Error::map_from(Box::new(e))),
    }
}

pub fn get_permissions(
    client: &mut Client,
    guild_id: String,
    role_ids: Vec<String>,
) -> Result<Vec<String>, Error> {
    client
        .query(
            "SELECT permission FROM permissions WHERE guild_id=$1 AND role_id IN $2",
            &[&guild_id, &role_ids],
        )
        .map(|rows| rows.iter().map(|row| row.get(0)).collect())
        .map_err(|e| Error::map_from(Box::new(e)))
}

pub fn has_permission(
    client: &mut Client,
    guild_id: String,
    role_ids: Vec<String>,
    permission: String,
) -> Result<bool, Error> {
    client
        .query(
            "SELECT 1 FROM permissions WHERE guild_id=$1 AND role_id IN $2 AND permission=$3 LIMIT 1",
            &[&guild_id, &role_ids, &permission],
        )
        .map(|rows| !rows.is_empty())
        .map_err(|e| Error::map_from(Box::new(e)))
}

pub fn existing_groups(client: &mut Client, guild_ids: Vec<&String>) -> Result<Vec<String>, Error> {
    let stmt = client
        .prepare_typed(
            "SELECT guild_id FROM groups WHERE guild_id = ANY($1)",
            &[Type::VARCHAR_ARRAY],
        )
        .map_err(|e| Error::map_from(Box::new(e)))?;
    client
        .query(&stmt, &[&guild_ids])
        .map(|rows| rows.iter().map(|row| row.get(0)).collect())
        .map_err(|e| Error::map_from(Box::new(e)))
}
