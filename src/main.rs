use actix_files as fs;
use actix_web::{middleware, web, App, HttpServer};
use std::sync::Mutex;

mod api;

async fn not_found() -> actix_web::Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("./public/404.html")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info,reqwest=trace");
    env_logger::init();

    let client_id = std::env::get("CLIENT_ID");
    let client_secret = std::env::get("CLIENT_SECRET");
    let http_client = reqwest::Client::builder()
        .user_agent("Rusty Planner")
        .connection_verbose(true)
        .build()
        .unwrap();

    HttpServer::new(move || {
        // TODO: Swap to async client
        let pg_client = Mutex::new(
            postgres::Client::connect("host=localhost user=postgres", postgres::NoTls).unwrap(),
        );

        App::new()
            .wrap(middleware::Logger::default())
            .default_service(web::route().to(not_found))
            .service(
                web::scope("/api")
                    .app_data(api::AppData {
                        pg_client,
                        http_client: http_client.clone(),
                        client_id,
                        client_secret,
                    })
                    .default_service(
                        web::route().to(|| api::simple_response(404, "Not found".to_string())),
                    )
                    .configure(api::configure),
            )
            .service(
                fs::Files::new("/", "./public")
                    .use_last_modified(true)
                    .prefer_utf8(true)
                    .redirect_to_slash_directory()
                    .index_file("index.html"),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
