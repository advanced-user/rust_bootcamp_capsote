use std::str::FromStr;
use actix::prelude::{Message, Recipient};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
    pub game_id: Uuid,
    pub self_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
    pub game_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientActorMessage {
    pub id: Uuid,
    pub msg: Msg,
    pub game_id: Uuid,
}

#[derive(PartialEq)]
pub enum Msg {
    Left,  // The player moves the character one unit to the left
    Right, // The player moves the character one unit to the right
    Hit,   // A player tries to hit another character
    Wait,  // The player does nothing
}

impl FromStr for Msg {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        chars.next();
        chars.next_back();

        let msg_str= chars.as_str();

        match msg_str {
            "Left" => Ok(Msg::Left),
            "Right" => Ok(Msg::Right),
            "Hit" => Ok(Msg::Hit),
            "Wait" => Ok(Msg::Wait),
            _ => {
                Err(())
            },
        }
    }
}