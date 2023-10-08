mod lobby;
mod ws;
use lobby::Lobby;
mod game;
mod messages;
mod start_connection;
mod errors;

use actix::Actor;
use start_connection::start_connection as start_connection_route;

use actix_web::web::Data;
use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let game_server = Lobby::default().start();

    HttpServer::new(move || {
        App::new()
            .service(start_connection_route)
            .app_data(Data::new(game_server.clone()))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
