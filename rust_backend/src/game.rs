use crate::messages::{ClientActorMessage, Msg};
use std::collections::HashMap;
use serde::{Serialize};
use uuid::Uuid;

const START_LEFT_POSITION: PlayerPosition = PlayerPosition { y: 5f64, x: 20f64 };
const START_RIGHT_POSITION: PlayerPosition = PlayerPosition { y: 5f64, x: 300f64 };

#[derive(Serialize)]
pub struct PlayerPosition {
    x: f64,
    y: f64,
}

#[derive(Serialize)]
struct PlayerBody {
    height: i32,
    width: i32,
}

#[derive(Serialize)]
struct Player {
    health: i32,
    speed: f64,
    damage: i32,
    attack_zone: i32,
    body: PlayerBody,
    position: PlayerPosition,
}

impl Player {
    fn go_right(&mut self) {
        self.position.x += self.speed;
    }

    fn go_left(&mut self) {
        self.position.x -= self.speed;
    }

    fn take_damage(&mut self, damage: i32) {
        self.health -= damage;
    }

    fn is_alive(&self) -> bool {
        if self.health <= 0 {
            return false;
        }

        true
    }
}

#[derive(PartialEq)]
#[derive(Serialize)]
enum GameState {
    NotStarted,
    InProgress,
    GameOver,
}

#[derive(Serialize)]
enum Winner {
    Id(Uuid),
    None,
}

#[derive(Serialize)]
pub struct Game {
    players: HashMap<Uuid, Player>,
    state: GameState,
    winner: Winner,
}

impl Game {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            state: GameState::NotStarted,
            winner: Winner::None,
        }
    }

    pub fn add_player(&mut self, player_id: Uuid) -> Result<(), ()> {
        if self.players.len() >= 2 {
            return Err(());
        }

        let player_position = if self.players.len() == 0 {
            START_LEFT_POSITION
        } else {
            START_RIGHT_POSITION
        };

        // In the future we will load from the database
        let new_player = Player {
            health: 1000,
            speed: 40f64,
            damage: 100,
            attack_zone: 30,
            body: PlayerBody {
                height: 10,
                width: 5,
            },
            position: player_position,
        };

        self.players.insert(player_id, new_player);

        if self.players.len() == 2 {
            self.state = GameState::InProgress;
        }

        Ok(())
    }

    pub fn handle_msg(&mut self, msg: &ClientActorMessage) {
        if self.state == GameState::GameOver || self.state == GameState::NotStarted {
            return;
        }

        let opponent_id = self.get_opponent_id(&msg.id);
        match msg.msg {
            Msg::Hit => self.handle_attack(&msg.id, &opponent_id),
            Msg::Wait => {},
            _ => self.handle_movement(&msg.id, &msg.msg),
        }
    }

    fn get_opponent_id(&self, player_id: &Uuid) -> Uuid {
        for (uuid, _) in &self.players {
            if uuid != player_id {
                return uuid.clone();
            }
        }

        panic!("Player not found")
    }

    pub fn handle_movement(&mut self, player_id: &Uuid, direction: &Msg) {
        let player = self.players.get_mut(player_id).expect("Player not found");
        if direction == &Msg::Left {
            player.go_left();
        } else {
            player.go_right();
        }
    }

    pub fn handle_attack(&mut self, attacker_id: &Uuid, def_id: &Uuid) {
        if self.in_attack_zone(attacker_id, def_id) {
            let attacker = self.players.get(attacker_id).expect("Player not found");
            let damage = attacker.damage;
            let def_player = self.players.get_mut(def_id).expect("Player not found");
            def_player.take_damage(damage);
            if !def_player.is_alive() {
                self.state = GameState::GameOver;
                self.winner = Winner::Id(attacker_id.clone());
            }
        }
    }

    fn in_attack_zone(&self, attacker_id: &Uuid, def_id: &Uuid) -> bool {
        let attacker = self.players.get(attacker_id).expect("Player not found");
        let def_player = self.players.get(def_id).expect("Player not found");

        let body_radius = def_player.body.width as f64 / 2f64;
        let left_border = def_player.position.x - body_radius;
        let right_border = def_player.position.x - body_radius;

        let left_attack_zone = attacker.position.x - attacker.attack_zone as f64;
        let right_attack_zone = attacker.position.x + attacker.attack_zone as f64;

        return if self.in_range(&right_attack_zone, &left_border, &right_border) {
            true
        } else if self.in_range(&left_attack_zone, &left_border, &right_border) {
            true
        } else {
            false
        };
    }

    fn in_range(&self, val: &f64, left_border: &f64, right_border: &f64) -> bool {
        if val >= left_border && val <= right_border {
            return true;
        }

        return false;
    }
}
