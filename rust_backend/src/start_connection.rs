use crate::lobby::Lobby;
use crate::ws::WsConn;
use actix::Addr;
use actix_web::{get, web::Data, web::Path, web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use uuid::Uuid;

#[get("/{game_id}")]
pub async fn start_connection(
    req: HttpRequest,
    stream: Payload,
    path: Path<Uuid>,
    srv: Data<Addr<Lobby>>,
) -> Result<HttpResponse, Error> {
    let game_id = path.into_inner();
    let ws = WsConn::new(game_id, srv.get_ref().clone());

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
