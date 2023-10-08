use crate::messages::{ClientActorMessage, Msg};
use std::collections::HashMap;
use serde::{Serialize};
use uuid::Uuid;
use crate::errors::{AppError, AppErrorType};

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

#[derive(PartialEq, Debug)]
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

    fn get_player_mut(&mut self, player_id: &Uuid) -> Result<&mut Player, AppError> {
        if let Some(player) =  self.players.get_mut(player_id) {
            Ok(player)
        } else {
            return Err(AppError {
                message: "Player not found".to_string(),
                error_type: AppErrorType::PlayerNotFoundError,
            })
        }
    }

    fn get_player(&self, player_id: &Uuid) -> Result<&Player, AppError> {
        if let Some(player) =  self.players.get(player_id) {
            Ok(player)
        } else {
            return Err(AppError {
                message: "Player not found".to_string(),
                error_type: AppErrorType::PlayerNotFoundError,
            })
        }
    }

    pub fn add_player(&mut self, player_id: Uuid) -> Result<(), AppError> {
        if self.players.len() >= 2 {
            return Err(AppError { message: "The number of players cannot exceed 2".to_string(), error_type: AppErrorType::NumberOfPlayersError });
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

    pub fn handle_msg(&mut self, msg: &ClientActorMessage) -> Result<(), AppError> {
        if self.state == GameState::GameOver || self.state == GameState::NotStarted {
            return Err(AppError {
                message: format!("Game state is {:?}", self.state),
                error_type: AppErrorType::GameStateError,
            });
        }

        let opponent_id = self.get_opponent_id(&msg.id)?;
        match msg.msg {
            Msg::Hit => self.handle_attack(&msg.id, &opponent_id)?,
            Msg::Wait => {},
            _ => self.handle_movement(&msg.id, &msg.msg)?,
        }

        Ok(())
    }

    fn get_opponent_id(&self, player_id: &Uuid) -> Result<Uuid, AppError> {
        for (uuid, _) in &self.players {
            if uuid != player_id {
                return Ok(uuid.clone());
            }
        }

        Err(AppError {
            message: "Opponent player not found".to_string(),
            error_type: AppErrorType::PlayerNotFoundError,
        })
    }

    pub fn handle_movement(&mut self, player_id: &Uuid, direction: &Msg) -> Result<(), AppError> {
        let player = self.get_player_mut(player_id)?;

        if direction == &Msg::Left {
            player.go_left();
        } else {
            player.go_right();
        }

        Ok(())
    }

    pub fn handle_attack(&mut self, attacker_id: &Uuid, def_id: &Uuid) -> Result<(), AppError> {
        if self.in_attack_zone(attacker_id, def_id)? {
            let attacker = self.get_player(attacker_id)?;
            let damage = attacker.damage;
            let def_player = self.get_player_mut(def_id)?;
            def_player.take_damage(damage);
            if !def_player.is_alive() {
                self.state = GameState::GameOver;
                self.winner = Winner::Id(attacker_id.clone());
            }
        }

        Ok(())
    }

    fn in_attack_zone(&self, attacker_id: &Uuid, def_id: &Uuid) -> Result<bool, AppError> {
        let attacker = self.get_player(attacker_id)?;
        let def_player = self.get_player(def_id)?;

        let body_radius = def_player.body.width as f64 / 2f64;
        let left_border = def_player.position.x - body_radius;
        let right_border = def_player.position.x - body_radius;

        let left_attack_zone = attacker.position.x - attacker.attack_zone as f64;
        let right_attack_zone = attacker.position.x + attacker.attack_zone as f64;

        return if self.in_range(&right_attack_zone, &left_border, &right_border) {
            Ok(true)
        } else if self.in_range(&left_attack_zone, &left_border, &right_border) {
            Ok(true)
        } else {
            Ok(false)
        };
    }

    fn in_range(&self, val: &f64, left_border: &f64, right_border: &f64) -> bool {
        if val >= left_border && val <= right_border {
            return true;
        }

        return false;
    }
}
