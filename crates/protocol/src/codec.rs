/// Generic Protocol Codec with Versioning
///
/// Этот модуль предоставляет универсальный кодек для сериализации и
/// десериализации коллекций элементов с поддержкой версионирования.
///
/// # Format
///
/// New format with version:
/// ```text
/// [VERSION:u8][COUNT:u32][TIMESTAMP:u32][ITEMS...]
/// ```
///
/// Legacy format (no version):
/// ```text
/// [COUNT:u32][TIMESTAMP:u32][ITEMS...]
/// ```
///
/// # Examples
///
/// ```rust
/// use protocol::{ProtocolCodec, EntityState, BinarySerialize};
///
/// let codec = ProtocolCodec::<EntityState>::new(1);
/// let entities = vec![
///     EntityState::new(1, 10.0, 20.0, 0.5, 0),
///     EntityState::new(2, 30.0, 40.0, 1.0, 1),
/// ];
///
/// // Encode with version
/// let buffer = codec.encode(&entities);
///
/// // Encode in legacy format (backward compatibility)
/// let legacy_buffer = codec.encode_legacy(&entities);
/// ```
use crate::traits::{BinarySerialize, BinaryDeserialize, DecodeError};
use std::marker::PhantomData;

/// Generic Protocol Codec
///
/// Универсальный кодек для сериализации и десериализации коллекций
/// элементов с поддержкой версионирования.
///
/// # Type Parameters
///
/// - `T`: Тип элементов, которые будут кодироваться/декодироваться.
///        Должен реализовывать `BinarySerialize` для кодирования и
///        `BinaryDeserialize` для декодирования.
///
/// # Examples
///
/// ```rust
/// use protocol::{ProtocolCodec, EntityState};
///
/// let codec = ProtocolCodec::<EntityState>::new(1);
/// let entities = vec![EntityState::new(1, 10.0, 20.0, 0.5, 0)];
/// let buffer = codec.encode(&entities);
/// ```
pub struct ProtocolCodec<T> {
    version: u8,
    _phantom: PhantomData<T>,
}

impl<T: BinarySerialize> ProtocolCodec<T> {
    /// Создать новый кодек с указанной версией
    ///
    /// # Parameters
    ///
    /// - `version`: Версия протокола (обычно 1 для нового формата)
    ///
    /// # Returns
    ///
    /// Новый экземпляр `ProtocolCodec<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use protocol::{ProtocolCodec, EntityState};
    ///
    /// let codec = ProtocolCodec::<EntityState>::new(1);
    /// ```
    pub fn new(version: u8) -> Self {
        Self {
            version,
            _phantom: PhantomData,
        }
    }

    /// Кодировать коллекцию элементов с версионированием
    ///
    /// Формат: `[VERSION:u8][COUNT:u32][TIMESTAMP:u32][ITEMS...]`
    ///
    /// # Parameters
    ///
    /// - `items`: Коллекция элементов для кодирования
    ///
    /// # Returns
    ///
    /// Бинарный буфер с закодированными данными
    ///
    /// # Examples
    ///
    /// ```rust
    /// use protocol::{ProtocolCodec, EntityState};
    ///
    /// let codec = ProtocolCodec::<EntityState>::new(1);
    /// let entities = vec![EntityState::new(1, 10.0, 20.0, 0.5, 0)];
    /// let buffer = codec.encode(&entities);
    ///
    /// assert_eq!(buffer[0], 1); // Version
    /// ```
    pub fn encode(&self, items: &[T]) -> Vec<u8> {
        let total_size = 1 + 4 + 4 + items.iter().map(|i| i.byte_size()).sum::<usize>();
        let mut buffer = Vec::with_capacity(total_size);

        // Версия протокола (1 байт)
        buffer.push(self.version);

        // Количество элементов (4 байта)
        buffer.extend_from_slice(&(items.len() as u32).to_le_bytes());

        // Timestamp (4 байта)
        let timestamp = current_timestamp();
        buffer.extend_from_slice(&timestamp.to_le_bytes());

        // Данные
        for item in items {
            item.write_to(&mut buffer);
        }

        buffer
    }

    /// Кодировать в старом формате (без версии) для backward compatibility
    ///
    /// Формат: `[COUNT:u32][TIMESTAMP:u32][ITEMS...]`
    ///
    /// # Parameters
    ///
    /// - `items`: Коллекция элементов для кодирования
    ///
    /// # Returns
    ///
    /// Бинарный буфер с закодированными данными (без версии)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use protocol::{ProtocolCodec, EntityState};
    ///
    /// let codec = ProtocolCodec::<EntityState>::new(1);
    /// let entities = vec![EntityState::new(1, 10.0, 20.0, 0.5, 0)];
    /// let buffer = codec.encode_legacy(&entities);
    ///
    /// // No version byte in legacy format
    /// assert_eq!(buffer.len(), 8 + 20); // header + 1 entity
    /// ```
    pub fn encode_legacy(&self, items: &[T]) -> Vec<u8> {
        let total_size = 4 + 4 + items.iter().map(|i| i.byte_size()).sum::<usize>();
        let mut buffer = Vec::with_capacity(total_size);

        // Количество элементов (4 байта)
        buffer.extend_from_slice(&(items.len() as u32).to_le_bytes());

        // Timestamp (4 байта)
        buffer.extend_from_slice(&0u32.to_le_bytes());

        // Данные
        for item in items {
            item.write_to(&mut buffer);
        }

        buffer
    }
}

impl<T: BinaryDeserialize> ProtocolCodec<T> {
    /// Декодировать коллекцию элементов
    ///
    /// Ожидает формат: `[VERSION:u8][COUNT:u32][TIMESTAMP:u32][ITEMS...]`
    ///
    /// # Parameters
    ///
    /// - `buffer`: Бинарный буфер с закодированными данными
    ///
    /// # Returns
    ///
    /// - `Ok(DecodedMessage<T>)`: Успешно декодированное сообщение
    /// - `Err(DecodeError)`: Ошибка декодирования
    ///
    /// # Errors
    ///
    /// - `DecodeError::BufferTooShort`: Буфер слишком короткий
    /// - `DecodeError::InvalidData`: Несовпадение версии или некорректные данные
    ///
    /// # Examples
    ///
    /// ```rust
    /// use protocol::{ProtocolCodec, EntityState};
    ///
    /// let codec = ProtocolCodec::<EntityState>::new(1);
    /// let entities = vec![EntityState::new(1, 10.0, 20.0, 0.5, 0)];
    /// let buffer = codec.encode(&entities);
    ///
    /// let decoded = codec.decode(&buffer).unwrap();
    /// assert_eq!(decoded.version, 1);
    /// assert_eq!(decoded.items.len(), 1);
    /// ```
    pub fn decode(&self, buffer: &[u8]) -> Result<DecodedMessage<T>, DecodeError> {
        if buffer.len() < 9 {
            return Err(DecodeError::BufferTooShort {
                expected: 9,
                actual: buffer.len(),
            });
        }

        // Версия
        let version = buffer[0];
        if version != self.version {
            return Err(DecodeError::InvalidData(format!(
                "Version mismatch: expected {}, got {}",
                self.version, version
            )));
        }

        // Количество элементов
        let count = u32::from_le_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]) as usize;

        // Timestamp
        let timestamp = u32::from_le_bytes([buffer[5], buffer[6], buffer[7], buffer[8]]);

        // Декодировать элементы
        let mut items = Vec::with_capacity(count);
        let mut offset = 9;

        for _ in 0..count {
            let item = T::read_from(buffer, offset)?;
            offset += T::byte_size();
            items.push(item);
        }

        Ok(DecodedMessage {
            version,
            timestamp,
            items,
        })
    }
}

/// Декодированное сообщение
///
/// Содержит версию протокола, timestamp и коллекцию декодированных элементов.
///
/// # Type Parameters
///
/// - `T`: Тип декодированных элементов
///
/// # Fields
///
/// - `version`: Версия протокола
/// - `timestamp`: Timestamp сообщения
/// - `items`: Коллекция декодированных элементов
pub struct DecodedMessage<T> {
    pub version: u8,
    pub timestamp: u32,
    pub items: Vec<T>,
}

/// Получить текущий timestamp
///
/// Возвращает количество миллисекунд с UNIX epoch как u32.
///
/// # Returns
///
/// Текущий timestamp в миллисекундах
fn current_timestamp() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::EntityState;

    #[test]
    fn test_codec_new() {
        let codec = ProtocolCodec::<EntityState>::new(1);
        assert_eq!(codec.version, 1);
    }

    #[test]
    fn test_encode_empty() {
        let codec = ProtocolCodec::<EntityState>::new(1);
        let entities = vec![];
        let buffer = codec.encode(&entities);

        assert_eq!(buffer.len(), 9); // 1 (version) + 4 (count) + 4 (timestamp)
        assert_eq!(buffer[0], 1); // Version
        assert_eq!(
            u32::from_le_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]),
            0
        ); // Count
    }

    #[test]
    fn test_encode_single_entity() {
        let codec = ProtocolCodec::<EntityState>::new(1);
        let entities = vec![EntityState::new(1, 100.0, 200.0, 1.5, 0)];
        let buffer = codec.encode(&entities);

        assert_eq!(buffer.len(), 9 + 20); // header + 1 entity

        // Check version
        assert_eq!(buffer[0], 1);

        // Check count
        let count = u32::from_le_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]);
        assert_eq!(count, 1);

        // Check entity id
        let id = u32::from_le_bytes([buffer[9], buffer[10], buffer[11], buffer[12]]);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_encode_multiple_entities() {
        let codec = ProtocolCodec::<EntityState>::new(1);
        let entities = vec![
            EntityState::new(1, 10.0, 20.0, 0.5, 0),
            EntityState::new(2, 30.0, 40.0, 1.0, 1),
            EntityState::new(3, 50.0, 60.0, 1.5, 2),
        ];
        let buffer = codec.encode(&entities);

        assert_eq!(buffer.len(), 9 + 60); // header + 3 entities

        // Check version
        assert_eq!(buffer[0], 1);

        // Check count
        let count = u32::from_le_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_encode_legacy_empty() {
        let codec = ProtocolCodec::<EntityState>::new(1);
        let entities = vec![];
        let buffer = codec.encode_legacy(&entities);

        assert_eq!(buffer.len(), 8); // 4 (count) + 4 (timestamp), no version
        assert_eq!(
            u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
            0
        ); // Count
    }

    #[test]
    fn test_encode_legacy_single_entity() {
        let codec = ProtocolCodec::<EntityState>::new(1);
        let entities = vec![EntityState::new(1, 100.0, 200.0, 1.5, 0)];
        let buffer = codec.encode_legacy(&entities);

        assert_eq!(buffer.len(), 8 + 20); // header + 1 entity

        // Check count
        let count = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        assert_eq!(count, 1);

        // Check entity id
        let id = u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_decode_empty() {
        let codec = ProtocolCodec::<EntityState>::new(1);
        let entities = vec![];
        let buffer = codec.encode(&entities);

        let decoded = codec.decode(&buffer).unwrap();

        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.items.len(), 0);
    }

    #[test]
    fn test_decode_single_entity() {
        let codec = ProtocolCodec::<EntityState>::new(1);
        let entities = vec![EntityState::new(42, 100.5, 200.75, 1.57, 3)];
        let buffer = codec.encode(&entities);

        let decoded = codec.decode(&buffer).unwrap();

        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.items.len(), 1);
        assert_eq!(decoded.items[0].id, 42);
        assert!((decoded.items[0].x - 100.5).abs() < 0.001);
        assert!((decoded.items[0].y - 200.75).abs() < 0.001);
        assert!((decoded.items[0].rotation - 1.57).abs() < 0.001);
        assert_eq!(decoded.items[0].costume_id, 3);
    }

    #[test]
    fn test_decode_multiple_entities() {
        let codec = ProtocolCodec::<EntityState>::new(1);
        let entities = vec![
            EntityState::new(1, 10.0, 20.0, 0.5, 0),
            EntityState::new(2, 30.0, 40.0, 1.0, 1),
            EntityState::new(3, 50.0, 60.0, 1.5, 2),
        ];
        let buffer = codec.encode(&entities);

        let decoded = codec.decode(&buffer).unwrap();

        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.items.len(), 3);
        assert_eq!(decoded.items[0].id, 1);
        assert_eq!(decoded.items[1].id, 2);
        assert_eq!(decoded.items[2].id, 3);
    }

    #[test]
    fn test_decode_buffer_too_short() {
        let codec = ProtocolCodec::<EntityState>::new(1);
        let buffer = vec![1, 2, 3]; // Only 3 bytes

        let result = codec.decode(&buffer);

        assert!(result.is_err());
        match result {
            Err(DecodeError::BufferTooShort { expected, actual }) => {
                assert_eq!(expected, 9);
                assert_eq!(actual, 3);
            }
            _ => panic!("Expected BufferTooShort error"),
        }
    }

    #[test]
    fn test_decode_version_mismatch() {
        let codec_v1 = ProtocolCodec::<EntityState>::new(1);
        let codec_v2 = ProtocolCodec::<EntityState>::new(2);

        let entities = vec![EntityState::new(1, 10.0, 20.0, 0.5, 0)];
        let buffer = codec_v1.encode(&entities);

        let result = codec_v2.decode(&buffer);

        assert!(result.is_err());
        match result {
            Err(DecodeError::InvalidData(msg)) => {
                assert!(msg.contains("Version mismatch"));
                assert!(msg.contains("expected 2"));
                assert!(msg.contains("got 1"));
            }
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_roundtrip() {
        let codec = ProtocolCodec::<EntityState>::new(1);
        let original = vec![
            EntityState::new(1, 10.0, 20.0, 0.5, 0),
            EntityState::new(2, 30.0, 40.0, 1.0, 1),
            EntityState::new(3, 50.0, 60.0, 1.5, 2),
        ];

        let buffer = codec.encode(&original);
        let decoded = codec.decode(&buffer).unwrap();

        assert_eq!(decoded.items.len(), original.len());
        for (orig, dec) in original.iter().zip(decoded.items.iter()) {
            assert_eq!(orig.id, dec.id);
            assert!((orig.x - dec.x).abs() < 0.001);
            assert!((orig.y - dec.y).abs() < 0.001);
            assert!((orig.rotation - dec.rotation).abs() < 0.001);
            assert_eq!(orig.costume_id, dec.costume_id);
        }
    }

    #[test]
    fn test_timestamp_is_set() {
        let codec = ProtocolCodec::<EntityState>::new(1);
        let entities = vec![EntityState::new(1, 10.0, 20.0, 0.5, 0)];
        let buffer = codec.encode(&entities);

        let decoded = codec.decode(&buffer).unwrap();

        // Timestamp should be non-zero (current time)
        assert!(decoded.timestamp > 0);
    }

    #[test]
    fn test_different_versions() {
        let codec_v1 = ProtocolCodec::<EntityState>::new(1);
        let codec_v5 = ProtocolCodec::<EntityState>::new(5);

        let entities = vec![EntityState::new(1, 10.0, 20.0, 0.5, 0)];

        let buffer_v1 = codec_v1.encode(&entities);
        let buffer_v5 = codec_v5.encode(&entities);

        assert_eq!(buffer_v1[0], 1);
        assert_eq!(buffer_v5[0], 5);

        // Each codec can decode its own version
        assert!(codec_v1.decode(&buffer_v1).is_ok());
        assert!(codec_v5.decode(&buffer_v5).is_ok());

        // But not the other version
        assert!(codec_v1.decode(&buffer_v5).is_err());
        assert!(codec_v5.decode(&buffer_v1).is_err());
    }
}
