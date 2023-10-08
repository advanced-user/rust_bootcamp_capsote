#[derive(Debug)]
pub enum AppErrorType {
    GameStateError,
    NumberOfPlayersError,
    PlayerNotFoundError,
}

pub struct AppError {
    pub message: String,
    pub error_type: AppErrorType,
}