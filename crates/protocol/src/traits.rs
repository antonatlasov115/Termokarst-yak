/// Binary Protocol Traits
///
/// Этот модуль определяет traits для сериализации и десериализации
/// произвольных типов в бинарный формат. Позволяет легко добавлять
/// новые типы сообщений без изменения core кода.
///
/// # Examples
///
/// ```rust
/// use protocol::{BinarySerialize, BinaryDeserialize, DecodeError};
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct MyMessage {
///     id: u32,
///     value: f32,
/// }
///
/// impl BinarySerialize for MyMessage {
///     fn write_to(&self, buffer: &mut Vec<u8>) {
///         buffer.extend_from_slice(&self.id.to_le_bytes());
///         buffer.extend_from_slice(&self.value.to_le_bytes());
///     }
///     
///     fn byte_size(&self) -> usize {
///         8 // 4 bytes (u32) + 4 bytes (f32)
///     }
/// }
///
/// impl BinaryDeserialize for MyMessage {
///     fn read_from(buffer: &[u8], offset: usize) -> Result<Self, DecodeError> {
///         if buffer.len() < offset + 8 {
///             return Err(DecodeError::BufferTooShort {
///                 expected: offset + 8,
///                 actual: buffer.len(),
///             });
///         }
///         
///         let id = u32::from_le_bytes([
///             buffer[offset], buffer[offset + 1],
///             buffer[offset + 2], buffer[offset + 3]
///         ]);
///         let value = f32::from_le_bytes([
///             buffer[offset + 4], buffer[offset + 5],
///             buffer[offset + 6], buffer[offset + 7]
///         ]);
///         
///         Ok(Self { id, value })
///     }
///     
///     fn byte_size() -> usize {
///         8
///     }
/// }
/// ```
use std::fmt;

/// Trait для типов, которые можно сериализовать в бинарный формат
///
/// Этот trait позволяет любому типу определить, как он должен быть
/// записан в бинарный буфер. Используется generic `ProtocolCodec`
/// для кодирования коллекций элементов.
///
/// # Требования
///
/// - Все multi-byte значения должны использовать little-endian byte order
/// - `byte_size()` должен возвращать точный размер, который будет записан
/// - `write_to()` должен записывать ровно `byte_size()` байт
///
/// # Examples
///
/// ```rust
/// use protocol::BinarySerialize;
///
/// struct Point {
///     x: f32,
///     y: f32,
/// }
///
/// impl BinarySerialize for Point {
///     fn write_to(&self, buffer: &mut Vec<u8>) {
///         buffer.extend_from_slice(&self.x.to_le_bytes());
///         buffer.extend_from_slice(&self.y.to_le_bytes());
///     }
///     
///     fn byte_size(&self) -> usize {
///         8 // 2 * 4 bytes (f32)
///     }
/// }
/// ```
pub trait BinarySerialize {
    /// Записать данные в буфер
    ///
    /// Этот метод должен записать все поля типа в буфер в определённом
    /// порядке, используя little-endian byte order для multi-byte значений.
    ///
    /// # Parameters
    ///
    /// - `buffer`: Буфер, в который записываются данные
    ///
    /// # Postconditions
    ///
    /// - Буфер увеличивается на `self.byte_size()` байт
    /// - Все данные записаны в little-endian формате
    fn write_to(&self, buffer: &mut Vec<u8>);

    /// Размер в байтах
    ///
    /// Возвращает количество байт, которое будет записано методом `write_to()`.
    ///
    /// # Returns
    ///
    /// Размер в байтах (должен быть константой для данного типа)
    fn byte_size(&self) -> usize;
}

/// Trait для типов, которые можно десериализовать из бинарного формата
///
/// Этот trait позволяет любому типу определить, как он должен быть
/// прочитан из бинарного буфера. Используется generic `ProtocolCodec`
/// для декодирования коллекций элементов.
///
/// # Требования
///
/// - Все multi-byte значения должны читаться в little-endian byte order
/// - `byte_size()` должен возвращать точный размер, который будет прочитан
/// - `read_from()` должен читать ровно `byte_size()` байт
/// - `read_from()` должен проверять размер буфера перед чтением
///
/// # Examples
///
/// ```rust
/// use protocol::{BinaryDeserialize, DecodeError};
///
/// struct Point {
///     x: f32,
///     y: f32,
/// }
///
/// impl BinaryDeserialize for Point {
///     fn read_from(buffer: &[u8], offset: usize) -> Result<Self, DecodeError> {
///         if buffer.len() < offset + 8 {
///             return Err(DecodeError::BufferTooShort {
///                 expected: offset + 8,
///                 actual: buffer.len(),
///             });
///         }
///         
///         let x = f32::from_le_bytes([
///             buffer[offset], buffer[offset + 1],
///             buffer[offset + 2], buffer[offset + 3]
///         ]);
///         let y = f32::from_le_bytes([
///             buffer[offset + 4], buffer[offset + 5],
///             buffer[offset + 6], buffer[offset + 7]
///         ]);
///         
///         Ok(Self { x, y })
///     }
///     
///     fn byte_size() -> usize {
///         8
///     }
/// }
/// ```
pub trait BinaryDeserialize: Sized {
    /// Прочитать данные из буфера
    ///
    /// Этот метод должен прочитать все поля типа из буфера, начиная с
    /// указанного offset, используя little-endian byte order.
    ///
    /// # Parameters
    ///
    /// - `buffer`: Буфер, из которого читаются данные
    /// - `offset`: Начальная позиция в буфере
    ///
    /// # Returns
    ///
    /// - `Ok(Self)`: Успешно десериализованный объект
    /// - `Err(DecodeError)`: Ошибка декодирования
    ///
    /// # Errors
    ///
    /// - `DecodeError::BufferTooShort`: Буфер слишком короткий
    /// - `DecodeError::InvalidData`: Данные некорректны
    fn read_from(buffer: &[u8], offset: usize) -> Result<Self, DecodeError>;

    /// Размер в байтах
    ///
    /// Возвращает количество байт, которое будет прочитано методом `read_from()`.
    ///
    /// # Returns
    ///
    /// Размер в байтах (должен быть константой для данного типа)
    fn byte_size() -> usize;
}

/// Ошибки декодирования
///
/// Этот enum описывает все возможные ошибки, которые могут возникнуть
/// при декодировании бинарных данных.
#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
    /// Буфер слишком короткий
    ///
    /// Возникает, когда размер буфера меньше, чем требуется для
    /// декодирования данных.
    BufferTooShort {
        /// Ожидаемый размер буфера
        expected: usize,
        /// Фактический размер буфера
        actual: usize,
    },

    /// Некорректные данные
    ///
    /// Возникает, когда данные в буфере не соответствуют ожидаемому формату.
    InvalidData(String),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::BufferTooShort { expected, actual } => {
                write!(
                    f,
                    "Buffer too short: expected at least {} bytes, got {}",
                    expected, actual
                )
            }
            DecodeError::InvalidData(msg) => {
                write!(f, "Invalid data: {}", msg)
            }
        }
    }
}

impl std::error::Error for DecodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    // Тестовый тип для проверки traits
    #[derive(Debug, Clone, PartialEq)]
    struct TestMessage {
        id: u32,
        value: f32,
    }

    impl BinarySerialize for TestMessage {
        fn write_to(&self, buffer: &mut Vec<u8>) {
            buffer.extend_from_slice(&self.id.to_le_bytes());
            buffer.extend_from_slice(&self.value.to_le_bytes());
        }

        fn byte_size(&self) -> usize {
            8
        }
    }

    impl BinaryDeserialize for TestMessage {
        fn read_from(buffer: &[u8], offset: usize) -> Result<Self, DecodeError> {
            if buffer.len() < offset + 8 {
                return Err(DecodeError::BufferTooShort {
                    expected: offset + 8,
                    actual: buffer.len(),
                });
            }

            let id = u32::from_le_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]);
            let value = f32::from_le_bytes([
                buffer[offset + 4],
                buffer[offset + 5],
                buffer[offset + 6],
                buffer[offset + 7],
            ]);

            Ok(Self { id, value })
        }

        fn byte_size() -> usize {
            8
        }
    }

    #[test]
    fn test_binary_serialize_write_to() {
        let msg = TestMessage {
            id: 42,
            value: 3.14,
        };

        let mut buffer = Vec::new();
        msg.write_to(&mut buffer);

        assert_eq!(buffer.len(), 8);
        assert_eq!(u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]), 42);
    }

    #[test]
    fn test_binary_serialize_byte_size() {
        let msg = TestMessage {
            id: 42,
            value: 3.14,
        };

        assert_eq!(msg.byte_size(), 8);
    }

    #[test]
    fn test_binary_deserialize_read_from() {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&42u32.to_le_bytes());
        buffer.extend_from_slice(&3.14f32.to_le_bytes());

        let msg = TestMessage::read_from(&buffer, 0).unwrap();

        assert_eq!(msg.id, 42);
        assert!((msg.value - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_binary_deserialize_buffer_too_short() {
        let buffer = vec![1, 2, 3]; // Только 3 байта

        let result = TestMessage::read_from(&buffer, 0);

        assert!(result.is_err());
        match result {
            Err(DecodeError::BufferTooShort { expected, actual }) => {
                assert_eq!(expected, 8);
                assert_eq!(actual, 3);
            }
            _ => panic!("Expected BufferTooShort error"),
        }
    }

    #[test]
    fn test_decode_error_display() {
        let error = DecodeError::BufferTooShort {
            expected: 10,
            actual: 5,
        };

        assert_eq!(
            error.to_string(),
            "Buffer too short: expected at least 10 bytes, got 5"
        );

        let error = DecodeError::InvalidData("test error".to_string());
        assert_eq!(error.to_string(), "Invalid data: test error");
    }
}
