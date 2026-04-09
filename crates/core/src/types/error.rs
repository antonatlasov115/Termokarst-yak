//! Типы ошибок

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThermokarstError {
    #[error("Неверные параметры: {0}")]
    InvalidParameters(String),

    #[error("Ошибка расчета: {0}")]
    CalculationError(String),

    #[error("Ошибка симуляции: {0}")]
    SimulationError(String),

    #[error("Ошибка ввода-вывода: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Ошибка сериализации: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ThermokarstError>;
