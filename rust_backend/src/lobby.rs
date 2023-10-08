use crate::messages::{ClientActorMessage, Connect, Disconnect, WsMessage};
use actix::prelude::{Actor, Context, Handler, Recipient};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use crate::game::{Game};

type Socket = Recipient<WsMessage>;

pub struct Lobby {
    sessions: HashMap<Uuid, Socket>,
    games_users_ids: HashMap<Uuid, HashSet<Uuid>>, // game id  to list of users id
    games: HashMap<Uuid, Game>,
}

impl Default for Lobby {
    fn default() -> Lobby {
        Lobby {
            sessions: HashMap::new(),
            games_users_ids: HashMap::new(),
            games: HashMap::default(),
        }
    }
}

impl Lobby {
    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage(message.to_owned()));
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.sessions.remove(&msg.id).is_some() {
            self.games_users_ids
                .get(&msg.game_id)
                .unwrap()
                .iter()
                .filter(|conn_id| *conn_id.to_owned() != msg.id)
                .for_each(|user_id| {
                    self.send_message(&format!("{} disconnected.", &msg.id), user_id)
                });
            if let Some(lobby) = self.games_users_ids.get_mut(&msg.game_id) {
                if lobby.len() > 1 {
                    lobby.remove(&msg.id);
                } else {
                    self.games_users_ids.remove(&msg.game_id);
                    self.games.remove(&msg.game_id);
                }
            }
        }
    }
}

impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        if let Some(game) = self.games.get_mut(&msg.game_id) {
            match game.add_player(msg.self_id) {
                Ok(_) => {}
                Err(_) => {
                    let _ = msg.addr.do_send(WsMessage("Unable to connect to the game".to_string()));
                    return;
                }
            }
        } else {
            match self.games
                .entry(msg.game_id)
                .or_insert_with(|| Game::new())
                .add_player(msg.self_id) {
                Ok(_) => {}
                Err(err) => {
                    let _ = msg.addr.do_send(WsMessage(err.message));
                    return;
                }
            }
        }

        self.games_users_ids
            .entry(msg.game_id)
            .or_insert_with(HashSet::new)
            .insert(msg.self_id);

        self.games_users_ids
            .get(&msg.game_id)
            .unwrap()
            .iter()
            .filter(|conn_id| *conn_id.to_owned() != msg.self_id)
            .for_each(|conn_id| {
                self.send_message(&format!("{} just joined!", msg.self_id), conn_id)
            });

        self.sessions.insert(msg.self_id, msg.addr);

        self.send_message(&format!("your id is {}", msg.self_id), &msg.self_id);
    }
}

impl Handler<ClientActorMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let game = self.games.get_mut(&msg.game_id).unwrap();
        let response;

        match game.handle_msg(&msg) {
            Ok(_) => response = serde_json::to_string(&game).unwrap(),
            Err(err) => response = err.message,
        }

        self.games_users_ids
            .get(&msg.game_id)
            .unwrap()
            .iter()
            .for_each(|client| self.send_message(&response, client));
    }
}
