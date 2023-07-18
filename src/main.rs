use actix_cors::Cors;
use actix_web::{
    http, middleware,
    web::{self, Data},
    App, HttpServer,
};
use args::*;
use clap::Parser;
use state::*;
use std::sync::Arc;
use subxt::{client::OnlineClient, PolkadotConfig};
use util::url_to_string;

mod args;
mod contract;
mod state;
mod types;
mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let args = Args::parse();

    let api = OnlineClient::<PolkadotConfig>::from_url(url_to_string(args.node_server))
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let state = AppState { api: Arc::new(api) };

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8080")
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().starts_with(b"http://localhost")
            })
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .app_data(Data::new(state.clone()))
            .route(
                "fula/contract/mint",
                web::post().to(contract::contract_mint_to),
            )
            .route(
                "fula/contract/supply",
                web::post().to(contract::contract_total_supply),
            )
            .route(
                "fula/contract/allowance",
                web::post().to(contract::contract_allowance),
            )
            .route(
                "fula/contract/increase_allowance",
                web::post().to(contract::contract_increase_allowance),
            )
            .route(
                "fula/contract/decrease_allowance",
                web::post().to(contract::contract_decrease_allowance),
            )
            .route(
                "fula/contract/burn",
                web::post().to(contract::contract_burn_from),
            )
            .route(
                "fula/contract/transfer",
                web::post().to(contract::contract_transfer),
            )
    })
    .bind((args.listen.host_str().unwrap(), args.listen.port().unwrap()))?
    .run()
    .await
}
