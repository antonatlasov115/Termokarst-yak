// 📦 Protocol Crate - Бинарный протокол для передачи состояния
//
// Отвечает ТОЛЬКО за кодирование/декодирование данных
// Не знает про ECS, физику или AST

pub mod encoder;
pub mod traits;
pub mod codec;

// Re-export
pub use encoder::{BinaryEncoder, EntityState, ENTITY_SIZE, HEADER_SIZE};
pub use traits::{BinarySerialize, BinaryDeserialize, DecodeError};
pub use codec::{ProtocolCodec, DecodedMessage};
