/// 📦 Binary State Protocol v1.0
///
/// Формат: [HEADER][ENTITY_1][ENTITY_2]...[ENTITY_N]
///
/// HEADER (8 bytes):
///   - entity_count: u32 (4 bytes)
///   - timestamp: u32 (4 bytes)
///
/// ENTITY (20 bytes):
///   - id: u32 (4 bytes)
///   - x: f32 (4 bytes)
///   - y: f32 (4 bytes)
///   - rotation: f32 (4 bytes)
///   - costume_id: u32 (4 bytes)
use crate::traits::{BinarySerialize, BinaryDeserialize, DecodeError};

pub const HEADER_SIZE: usize = 8;
pub const ENTITY_SIZE: usize = 20;

/// 🔢 Закодировать состояние сущности в бинарный формат
#[derive(Debug, Clone)]
pub struct EntityState {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub costume_id: u32,
}

impl EntityState {
    pub fn new(id: u32, x: f32, y: f32, rotation: f32, costume_id: u32) -> Self {
        Self {
            id,
            x,
            y,
            rotation,
            costume_id,
        }
    }
}

/// Implementation of BinarySerialize trait for EntityState
///
/// **Validates: Requirements 1.3, 1.4**
///
/// Allows EntityState to be serialized using the new trait-based API.
/// All fields are written in little-endian format.
impl BinarySerialize for EntityState {
    fn write_to(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.id.to_le_bytes());
        buffer.extend_from_slice(&self.x.to_le_bytes());
        buffer.extend_from_slice(&self.y.to_le_bytes());
        buffer.extend_from_slice(&self.rotation.to_le_bytes());
        buffer.extend_from_slice(&self.costume_id.to_le_bytes());
    }

    fn byte_size(&self) -> usize {
        ENTITY_SIZE
    }
}

/// Implementation of BinaryDeserialize trait for EntityState
///
/// **Validates: Requirements 1.3, 1.4**
///
/// Allows EntityState to be deserialized using the new trait-based API.
/// All fields are read in little-endian format with buffer size validation.
impl BinaryDeserialize for EntityState {
    fn read_from(buffer: &[u8], offset: usize) -> Result<Self, DecodeError> {
        if buffer.len() < offset + ENTITY_SIZE {
            return Err(DecodeError::BufferTooShort {
                expected: offset + ENTITY_SIZE,
                actual: buffer.len(),
            });
        }

        let id = u32::from_le_bytes([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ]);
        let x = f32::from_le_bytes([
            buffer[offset + 4],
            buffer[offset + 5],
            buffer[offset + 6],
            buffer[offset + 7],
        ]);
        let y = f32::from_le_bytes([
            buffer[offset + 8],
            buffer[offset + 9],
            buffer[offset + 10],
            buffer[offset + 11],
        ]);
        let rotation = f32::from_le_bytes([
            buffer[offset + 12],
            buffer[offset + 13],
            buffer[offset + 14],
            buffer[offset + 15],
        ]);
        let costume_id = u32::from_le_bytes([
            buffer[offset + 16],
            buffer[offset + 17],
            buffer[offset + 18],
            buffer[offset + 19],
        ]);

        Ok(Self {
            id,
            x,
            y,
            rotation,
            costume_id,
        })
    }

    fn byte_size() -> usize {
        ENTITY_SIZE
    }
}

/// 📤 Encoder: Rust → Binary
pub struct BinaryEncoder;

impl BinaryEncoder {
    /// Закодировать список сущностей в бинарный буфер
    pub fn encode(entities: &[EntityState]) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(HEADER_SIZE + entities.len() * ENTITY_SIZE);

        // Записать заголовок
        buffer.extend_from_slice(&(entities.len() as u32).to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes()); // timestamp (пока 0)

        // Записать сущности
        for entity in entities {
            buffer.extend_from_slice(&entity.id.to_le_bytes());
            buffer.extend_from_slice(&entity.x.to_le_bytes());
            buffer.extend_from_slice(&entity.y.to_le_bytes());
            buffer.extend_from_slice(&entity.rotation.to_le_bytes());
            buffer.extend_from_slice(&entity.costume_id.to_le_bytes());
        }

        buffer
    }

    /// Закодировать в Vec<f32> (старый формат для обратной совместимости)
    pub fn encode_f32(entities: &[EntityState]) -> Vec<f32> {
        let mut buffer = Vec::with_capacity(entities.len() * 5);

        for entity in entities {
            buffer.push(entity.id as f32);
            buffer.push(entity.x);
            buffer.push(entity.y);
            buffer.push(entity.rotation);
            buffer.push(entity.costume_id as f32);
        }

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_empty() {
        let entities = vec![];
        let buffer = BinaryEncoder::encode(&entities);

        assert_eq!(buffer.len(), HEADER_SIZE);
        assert_eq!(
            u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
            0
        );
    }

    #[test]
    fn test_encode_single_entity() {
        let entities = vec![EntityState::new(1, 100.0, 200.0, 1.5, 0)];
        let buffer = BinaryEncoder::encode(&entities);

        assert_eq!(buffer.len(), HEADER_SIZE + ENTITY_SIZE);

        // Проверить заголовок
        let count = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        assert_eq!(count, 1);

        // Проверить данные сущности
        let id = u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_encode_f32_format() {
        let entities = vec![
            EntityState::new(1, 10.0, 20.0, 0.5, 0),
            EntityState::new(2, 30.0, 40.0, 1.0, 1),
        ];
        let buffer = BinaryEncoder::encode_f32(&entities);

        assert_eq!(buffer.len(), 10); // 2 entities * 5 floats
        assert_eq!(buffer[0], 1.0);
        assert_eq!(buffer[1], 10.0);
        assert_eq!(buffer[5], 2.0);
    }

    // Tests for BinarySerialize trait implementation
    #[test]
    fn test_entity_state_write_to() {
        let entity = EntityState::new(42, 100.5, 200.75, 1.57, 3);
        let mut buffer = Vec::new();

        entity.write_to(&mut buffer);

        assert_eq!(buffer.len(), 20);

        // Verify id
        let id = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        assert_eq!(id, 42);

        // Verify x
        let x = f32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
        assert!((x - 100.5).abs() < 0.001);

        // Verify y
        let y = f32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);
        assert!((y - 200.75).abs() < 0.001);

        // Verify rotation
        let rotation = f32::from_le_bytes([buffer[12], buffer[13], buffer[14], buffer[15]]);
        assert!((rotation - 1.57).abs() < 0.001);

        // Verify costume_id
        let costume_id = u32::from_le_bytes([buffer[16], buffer[17], buffer[18], buffer[19]]);
        assert_eq!(costume_id, 3);
    }

    #[test]
    fn test_entity_state_byte_size() {
        let entity = EntityState::new(1, 0.0, 0.0, 0.0, 0);
        assert_eq!(entity.byte_size(), 20);
        assert_eq!(entity.byte_size(), ENTITY_SIZE);
    }

    #[test]
    fn test_entity_state_write_to_multiple() {
        let entities = vec![
            EntityState::new(1, 10.0, 20.0, 0.5, 0),
            EntityState::new(2, 30.0, 40.0, 1.0, 1),
        ];

        let mut buffer = Vec::new();
        for entity in &entities {
            entity.write_to(&mut buffer);
        }

        assert_eq!(buffer.len(), 40); // 2 entities * 20 bytes

        // Verify first entity id
        let id1 = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        assert_eq!(id1, 1);

        // Verify second entity id
        let id2 = u32::from_le_bytes([buffer[20], buffer[21], buffer[22], buffer[23]]);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_entity_state_write_to_edge_cases() {
        // Test with zero values
        let entity_zero = EntityState::new(0, 0.0, 0.0, 0.0, 0);
        let mut buffer = Vec::new();
        entity_zero.write_to(&mut buffer);
        assert_eq!(buffer.len(), 20);

        // Test with max values
        let entity_max = EntityState::new(u32::MAX, f32::MAX, f32::MAX, f32::MAX, u32::MAX);
        let mut buffer = Vec::new();
        entity_max.write_to(&mut buffer);
        assert_eq!(buffer.len(), 20);

        // Test with negative floats
        let entity_neg = EntityState::new(1, -100.0, -200.0, -3.14, 0);
        let mut buffer = Vec::new();
        entity_neg.write_to(&mut buffer);
        assert_eq!(buffer.len(), 20);

        let x = f32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
        assert!((x - (-100.0)).abs() < 0.001);
    }

    // Tests for BinaryDeserialize trait implementation
    #[test]
    fn test_entity_state_read_from() {
        let entity = EntityState::new(42, 100.5, 200.75, 1.57, 3);
        let mut buffer = Vec::new();
        entity.write_to(&mut buffer);

        let deserialized = EntityState::read_from(&buffer, 0).unwrap();

        assert_eq!(deserialized.id, 42);
        assert!((deserialized.x - 100.5).abs() < 0.001);
        assert!((deserialized.y - 200.75).abs() < 0.001);
        assert!((deserialized.rotation - 1.57).abs() < 0.001);
        assert_eq!(deserialized.costume_id, 3);
    }

    #[test]
    fn test_entity_state_read_from_with_offset() {
        // Create a buffer with some padding before the entity data
        let mut buffer = vec![0xFF; 10]; // 10 bytes of padding
        let entity = EntityState::new(99, 50.0, 75.0, 2.5, 7);
        entity.write_to(&mut buffer);

        let deserialized = EntityState::read_from(&buffer, 10).unwrap();

        assert_eq!(deserialized.id, 99);
        assert!((deserialized.x - 50.0).abs() < 0.001);
        assert!((deserialized.y - 75.0).abs() < 0.001);
        assert!((deserialized.rotation - 2.5).abs() < 0.001);
        assert_eq!(deserialized.costume_id, 7);
    }

    #[test]
    fn test_entity_state_read_from_buffer_too_short() {
        let buffer = vec![1, 2, 3, 4, 5]; // Only 5 bytes, need 20

        let result = EntityState::read_from(&buffer, 0);

        assert!(result.is_err());
        match result {
            Err(DecodeError::BufferTooShort { expected, actual }) => {
                assert_eq!(expected, 20);
                assert_eq!(actual, 5);
            }
            _ => panic!("Expected BufferTooShort error"),
        }
    }

    #[test]
    fn test_entity_state_read_from_buffer_too_short_with_offset() {
        let buffer = vec![0; 25]; // 25 bytes total

        // Try to read at offset 10, which would require 30 bytes total
        let result = EntityState::read_from(&buffer, 10);

        assert!(result.is_err());
        match result {
            Err(DecodeError::BufferTooShort { expected, actual }) => {
                assert_eq!(expected, 30);
                assert_eq!(actual, 25);
            }
            _ => panic!("Expected BufferTooShort error"),
        }
    }

    #[test]
    fn test_entity_state_roundtrip() {
        let original = EntityState::new(123, 456.78, 901.23, 3.14159, 42);
        let mut buffer = Vec::new();
        original.write_to(&mut buffer);

        let deserialized = EntityState::read_from(&buffer, 0).unwrap();

        assert_eq!(deserialized.id, original.id);
        assert!((deserialized.x - original.x).abs() < 0.001);
        assert!((deserialized.y - original.y).abs() < 0.001);
        assert!((deserialized.rotation - original.rotation).abs() < 0.001);
        assert_eq!(deserialized.costume_id, original.costume_id);
    }

    #[test]
    fn test_entity_state_roundtrip_multiple() {
        let entities = vec![
            EntityState::new(1, 10.0, 20.0, 0.5, 0),
            EntityState::new(2, 30.0, 40.0, 1.0, 1),
            EntityState::new(3, 50.0, 60.0, 1.5, 2),
        ];

        let mut buffer = Vec::new();
        for entity in &entities {
            entity.write_to(&mut buffer);
        }

        let mut offset = 0;
        for original in &entities {
            let deserialized = EntityState::read_from(&buffer, offset).unwrap();
            assert_eq!(deserialized.id, original.id);
            assert!((deserialized.x - original.x).abs() < 0.001);
            assert!((deserialized.y - original.y).abs() < 0.001);
            assert!((deserialized.rotation - original.rotation).abs() < 0.001);
            assert_eq!(deserialized.costume_id, original.costume_id);
            offset += <EntityState as BinaryDeserialize>::byte_size();
        }
    }

    #[test]
    fn test_entity_state_roundtrip_edge_cases() {
        // Test with zero values
        let entity_zero = EntityState::new(0, 0.0, 0.0, 0.0, 0);
        let mut buffer = Vec::new();
        entity_zero.write_to(&mut buffer);
        let deserialized = EntityState::read_from(&buffer, 0).unwrap();
        assert_eq!(deserialized.id, 0);
        assert_eq!(deserialized.x, 0.0);
        assert_eq!(deserialized.y, 0.0);
        assert_eq!(deserialized.rotation, 0.0);
        assert_eq!(deserialized.costume_id, 0);

        // Test with negative floats
        let entity_neg = EntityState::new(1, -100.0, -200.0, -3.14, 0);
        let mut buffer = Vec::new();
        entity_neg.write_to(&mut buffer);
        let deserialized = EntityState::read_from(&buffer, 0).unwrap();
        assert_eq!(deserialized.id, 1);
        assert!((deserialized.x - (-100.0)).abs() < 0.001);
        assert!((deserialized.y - (-200.0)).abs() < 0.001);
        assert!((deserialized.rotation - (-3.14)).abs() < 0.001);
        assert_eq!(deserialized.costume_id, 0);

        // Test with max u32 values
        let entity_max = EntityState::new(u32::MAX, 1000.0, 2000.0, 6.28, u32::MAX);
        let mut buffer = Vec::new();
        entity_max.write_to(&mut buffer);
        let deserialized = EntityState::read_from(&buffer, 0).unwrap();
        assert_eq!(deserialized.id, u32::MAX);
        assert!((deserialized.x - 1000.0).abs() < 0.001);
        assert!((deserialized.y - 2000.0).abs() < 0.001);
        assert!((deserialized.rotation - 6.28).abs() < 0.001);
        assert_eq!(deserialized.costume_id, u32::MAX);
    }

    #[test]
    fn test_entity_state_byte_size_static() {
        assert_eq!(<EntityState as BinaryDeserialize>::byte_size(), 20);
        assert_eq!(<EntityState as BinaryDeserialize>::byte_size(), ENTITY_SIZE);
    }
}
