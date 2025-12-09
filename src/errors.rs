use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid opcode")]
    InvalidOpcode,
    #[error("invalid cartridge header size")]
    InvalidCartridgeHeaderSize,
    #[error("invalid NES file")]
    InvalidNesFile,
    #[error("invalid cartridge mapper id, only 0 is supported")]
    InvalidCartridgeMapper,
}
