use actix_web::{
    cookie::Cookie, guard, web, HttpMessage, HttpRequest, HttpResponse, Responder, Result,
};
use serde::Deserialize;
use std::sync::Mutex;
use uuid::Uuid;

mod database;
mod discord;
mod error;
mod response;
mod types;
use discord::API_ENDPOINT;
use error::Error;
use types::{CreateGroupRequest, User};

async fn oauth_redirect(req: HttpRequest) -> Result<impl Responder> {
    if let Some(data) = req.app_data::<AppData>() {
        let redirect_url = req.url_for_static("oauth_code")?;
        Ok(HttpResponse::TemporaryRedirect()
            .header(
                "Location",
                format!(
                    "{}/oauth2/authorize?{}",
                    API_ENDPOINT,
                    serde_urlencoded::to_string(&[
                        ("client_id", data.client_id),
                        ("redirect_uri", redirect_url.as_str()),
                        ("response_type", "code"),
                        ("scope", "identify guilds"),
                    ])?
                ),
            )
            .finish())
    } else {
        Ok(HttpResponse::InternalServerError().finish())
    }
}

#[derive(Deserialize)]
struct OauthCodeQuery {
    code: Option<String>,
}

async fn oauth_code(req: HttpRequest, query: web::Query<OauthCodeQuery>) -> Result<impl Responder> {
    if let Some(code) = &query.code {
        let app_data: &AppData = req.app_data().unwrap();

        let token =
            discord::oauth_token(app_data, &code, req.url_for_static("oauth_code")?.as_str())
                .await
                .map_err(Error::map_from)?;

        let user = discord::current_user(app_data, &token.access_token)
            .await
            .map_err(Error::map_from)?;

        let mut pg_client = app_data.pg_client.lock().unwrap();

        let uuid = database::create_session(&mut pg_client, &token, &user.into())?;
        Ok(HttpResponse::TemporaryRedirect()
            .cookie(
                Cookie::build("sess", uuid.to_simple().to_string())
                    .path("/")
                    .http_only(true)
                    .finish(),
            )
            .set_header("Location", "/")
            .finish())
    } else {
        Ok(HttpResponse::BadRequest().finish())
    }
}

async fn fetch_session(req: &HttpRequest) -> Result<database::Session> {
    (match req.cookie("sess") {
        Some(sess) => {
            let uuid = Uuid::parse_str(sess.value()).map_err(|e| Error::map_from(Box::new(e)))?;

            let app_data: &AppData = req.app_data().unwrap();
            let mut pg_client = app_data.pg_client.lock().unwrap();

            database::get_session(&mut pg_client, &uuid).map_err(|e| Error::map_from(Box::new(e)))
        }
        None => Err(Error::from_simple(response::SimpleResponse {
            code: 409,
            message: String::from("No session"),
        })),
    })
    .map_err(|e| e.into())
}

async fn create_group(
    req: HttpRequest,
    info: web::Json<CreateGroupRequest>,
) -> Result<impl Responder> {
    let session = fetch_session(&req).await?;

    Ok(HttpResponse::Ok().finish())
}

async fn get_user(req: HttpRequest) -> Result<impl Responder> {
    let session = fetch_session(&req).await?;
    let id = req.match_info().query("id");

    if id == "@me" {
        let user: User = session.into();
        Ok(HttpResponse::Ok().json(user))
    } else {
        Ok(simple_response(404, String::from("Not Found")).await)
    }
}

async fn get_elligible_guilds(req: HttpRequest) -> Result<impl Responder> {
    let session = fetch_session(&req).await?;
    let id = req.match_info().query("id");

    if id == "@me" {
        let app_data: &AppData = req.app_data().unwrap();
        match discord::current_user_guilds(&app_data, &session.token).await {
            Ok(guilds) => {
                let owned_guilds = guilds
                    .iter()
                    .filter(|g| g.owner)
                    .collect::<Vec<&discord::CurrentGuildsItem>>();

                let owned_guild_ids: Vec<&String> = owned_guilds.iter().map(|g| &g.id).collect();

                // TODO: This check is very slow, better to just decide which
                // are available client side and use database constraint to
                // prevent duplicate groups

                let mut pg_client = app_data.pg_client.lock().unwrap();
                let existing_groups = database::existing_groups(&mut pg_client, owned_guild_ids)?;
                drop(pg_client);

                println!("{:?}", existing_groups);

                Ok(HttpResponse::Ok().json(
                    owned_guilds
                        .iter()
                        .filter(|g| !existing_groups.contains(&g.id))
                        .collect::<Vec<&&discord::CurrentGuildsItem>>(),
                ))
            }
            Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
        }
    } else {
        Ok(simple_response(403, String::from("Forbidden")).await)
    }
}

pub async fn simple_response(code: u16, message: String) -> HttpResponse {
    HttpResponse::build(reqwest::StatusCode::from_u16(code).unwrap())
        .json(response::SimpleResponse { code, message })
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/ping")
            .guard(guard::Get())
            .to(|| simple_response(200, String::from("Pong!"))),
    )
    .service(
        web::resource("/oauth/redirect")
            .guard(guard::Get())
            .to(oauth_redirect),
    )
    .service(
        web::resource("/oauth/code")
            .name("oauth_code")
            .guard(guard::Get())
            .to(oauth_code),
    )
    .service(
        web::resource("/group")
            .guard(guard::Header("content-type", "application/json"))
            .route(web::post().to(create_group)),
    )
    .service(web::resource("/user/{id}").guard(guard::Get()).to(get_user))
    .service(
        web::resource("/user/{id}/guilds")
            .guard(guard::Get())
            .to(get_elligible_guilds),
    );
}

pub struct AppData {
    pub pg_client: Mutex<postgres::Client>,
    pub http_client: reqwest::Client,
    pub client_id: &'static str,
    pub client_secret: &'static str,
}
