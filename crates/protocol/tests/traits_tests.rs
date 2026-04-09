/// Unit tests for BinarySerialize and BinaryDeserialize traits
///
/// **Validates: Requirements 8.1**
///
/// These tests verify that the trait implementations work correctly:
/// - Roundtrip (serialize → deserialize) preserves data
/// - Buffer size validation works correctly
/// - byte_size() returns accurate values
use protocol::{BinarySerialize, BinaryDeserialize, DecodeError, EntityState};

#[test]
fn test_entity_state_roundtrip_basic() {
    // Test basic roundtrip: serialize → deserialize
    let original = EntityState::new(42, 100.5, 200.75, 1.57, 3);
    
    // Serialize
    let mut buffer = Vec::new();
    original.write_to(&mut buffer);
    
    // Deserialize
    let deserialized = EntityState::read_from(&buffer, 0).unwrap();
    
    // Verify all fields match
    assert_eq!(deserialized.id, original.id);
    assert!((deserialized.x - original.x).abs() < 0.001);
    assert!((deserialized.y - original.y).abs() < 0.001);
    assert!((deserialized.rotation - original.rotation).abs() < 0.001);
    assert_eq!(deserialized.costume_id, original.costume_id);
}

#[test]
fn test_entity_state_roundtrip_zero_values() {
    // Test roundtrip with all zero values
    let original = EntityState::new(0, 0.0, 0.0, 0.0, 0);
    
    let mut buffer = Vec::new();
    original.write_to(&mut buffer);
    
    let deserialized = EntityState::read_from(&buffer, 0).unwrap();
    
    assert_eq!(deserialized.id, 0);
    assert_eq!(deserialized.x, 0.0);
    assert_eq!(deserialized.y, 0.0);
    assert_eq!(deserialized.rotation, 0.0);
    assert_eq!(deserialized.costume_id, 0);
}

#[test]
fn test_entity_state_roundtrip_negative_floats() {
    // Test roundtrip with negative float values
    let original = EntityState::new(1, -100.0, -200.0, -3.14, 0);
    
    let mut buffer = Vec::new();
    original.write_to(&mut buffer);
    
    let deserialized = EntityState::read_from(&buffer, 0).unwrap();
    
    assert_eq!(deserialized.id, 1);
    assert!((deserialized.x - (-100.0)).abs() < 0.001);
    assert!((deserialized.y - (-200.0)).abs() < 0.001);
    assert!((deserialized.rotation - (-3.14)).abs() < 0.001);
    assert_eq!(deserialized.costume_id, 0);
}

#[test]
fn test_entity_state_roundtrip_max_values() {
    // Test roundtrip with maximum u32 values
    let original = EntityState::new(u32::MAX, 1000.0, 2000.0, 6.28, u32::MAX);
    
    let mut buffer = Vec::new();
    original.write_to(&mut buffer);
    
    let deserialized = EntityState::read_from(&buffer, 0).unwrap();
    
    assert_eq!(deserialized.id, u32::MAX);
    assert!((deserialized.x - 1000.0).abs() < 0.001);
    assert!((deserialized.y - 2000.0).abs() < 0.001);
    assert!((deserialized.rotation - 6.28).abs() < 0.001);
    assert_eq!(deserialized.costume_id, u32::MAX);
}

#[test]
fn test_entity_state_roundtrip_multiple_entities() {
    // Test roundtrip with multiple entities in sequence
    let entities = vec![
        EntityState::new(1, 10.0, 20.0, 0.5, 0),
        EntityState::new(2, 30.0, 40.0, 1.0, 1),
        EntityState::new(3, 50.0, 60.0, 1.5, 2),
    ];
    
    // Serialize all entities
    let mut buffer = Vec::new();
    for entity in &entities {
        entity.write_to(&mut buffer);
    }
    
    // Deserialize all entities
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
fn test_buffer_too_short_error_zero_bytes() {
    // Test error when buffer is completely empty
    let buffer = vec![];
    
    let result = EntityState::read_from(&buffer, 0);
    
    assert!(result.is_err());
    match result {
        Err(DecodeError::BufferTooShort { expected, actual }) => {
            assert_eq!(expected, 20);
            assert_eq!(actual, 0);
        }
        _ => panic!("Expected BufferTooShort error"),
    }
}

#[test]
fn test_buffer_too_short_error_partial_data() {
    // Test error when buffer has partial data (5 bytes, need 20)
    let buffer = vec![1, 2, 3, 4, 5];
    
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
fn test_buffer_too_short_error_one_byte_short() {
    // Test error when buffer is just one byte short
    let buffer = vec![0; 19]; // 19 bytes, need 20
    
    let result = EntityState::read_from(&buffer, 0);
    
    assert!(result.is_err());
    match result {
        Err(DecodeError::BufferTooShort { expected, actual }) => {
            assert_eq!(expected, 20);
            assert_eq!(actual, 19);
        }
        _ => panic!("Expected BufferTooShort error"),
    }
}

#[test]
fn test_buffer_too_short_error_with_offset() {
    // Test error when buffer is too short considering offset
    let buffer = vec![0; 25]; // 25 bytes total
    
    // Try to read at offset 10, which would require 30 bytes total
    let result = EntityState::read_from(&buffer, 10);
    
    assert!(result.is_err());
    match result {
        Err(DecodeError::BufferTooShort { expected, actual }) => {
            assert_eq!(expected, 30); // offset 10 + 20 bytes needed
            assert_eq!(actual, 25);
        }
        _ => panic!("Expected BufferTooShort error"),
    }
}

#[test]
fn test_buffer_too_short_error_large_offset() {
    // Test error when offset is beyond buffer size
    let buffer = vec![0; 10];
    
    let result = EntityState::read_from(&buffer, 100);
    
    assert!(result.is_err());
    match result {
        Err(DecodeError::BufferTooShort { expected, actual }) => {
            assert_eq!(expected, 120); // offset 100 + 20 bytes needed
            assert_eq!(actual, 10);
        }
        _ => panic!("Expected BufferTooShort error"),
    }
}

#[test]
fn test_byte_size_correctness_instance_method() {
    // Test that byte_size() returns correct value (instance method)
    let entity = EntityState::new(1, 0.0, 0.0, 0.0, 0);
    
    assert_eq!(entity.byte_size(), 20);
}

#[test]
fn test_byte_size_correctness_static_method() {
    // Test that byte_size() returns correct value (static method)
    assert_eq!(<EntityState as BinaryDeserialize>::byte_size(), 20);
}

#[test]
fn test_byte_size_matches_written_data() {
    // Test that byte_size() matches actual written data size
    let entity = EntityState::new(42, 100.5, 200.75, 1.57, 3);
    
    let mut buffer = Vec::new();
    entity.write_to(&mut buffer);
    
    // Verify that the actual written size matches byte_size()
    assert_eq!(buffer.len(), entity.byte_size());
    assert_eq!(buffer.len(), 20);
}

#[test]
fn test_byte_size_consistency_across_instances() {
    // Test that byte_size() is consistent across different instances
    let entity1 = EntityState::new(1, 10.0, 20.0, 0.5, 0);
    let entity2 = EntityState::new(999, -50.0, 100.0, 3.14, 42);
    let entity3 = EntityState::new(0, 0.0, 0.0, 0.0, 0);
    
    assert_eq!(entity1.byte_size(), 20);
    assert_eq!(entity2.byte_size(), 20);
    assert_eq!(entity3.byte_size(), 20);
    
    // All instances should have the same byte size
    assert_eq!(entity1.byte_size(), entity2.byte_size());
    assert_eq!(entity2.byte_size(), entity3.byte_size());
}

#[test]
fn test_write_to_buffer_growth() {
    // Test that write_to() correctly grows the buffer
    let entity = EntityState::new(42, 100.5, 200.75, 1.57, 3);
    
    let mut buffer = Vec::new();
    assert_eq!(buffer.len(), 0);
    
    entity.write_to(&mut buffer);
    assert_eq!(buffer.len(), 20);
    
    // Write another entity
    entity.write_to(&mut buffer);
    assert_eq!(buffer.len(), 40);
}

#[test]
fn test_read_from_with_valid_offset() {
    // Test reading from buffer with valid offset
    let mut buffer = vec![0xFF; 10]; // 10 bytes of padding
    let entity = EntityState::new(99, 50.0, 75.0, 2.5, 7);
    entity.write_to(&mut buffer);
    
    // Buffer now has 30 bytes: 10 padding + 20 entity data
    assert_eq!(buffer.len(), 30);
    
    // Read from offset 10
    let deserialized = EntityState::read_from(&buffer, 10).unwrap();
    
    assert_eq!(deserialized.id, 99);
    assert!((deserialized.x - 50.0).abs() < 0.001);
    assert!((deserialized.y - 75.0).abs() < 0.001);
    assert!((deserialized.rotation - 2.5).abs() < 0.001);
    assert_eq!(deserialized.costume_id, 7);
}

#[test]
fn test_little_endian_byte_order() {
    // Test that data is written in little-endian format
    let entity = EntityState::new(0x12345678, 0.0, 0.0, 0.0, 0);
    
    let mut buffer = Vec::new();
    entity.write_to(&mut buffer);
    
    // Verify little-endian byte order for id field
    // 0x12345678 in little-endian: 0x78, 0x56, 0x34, 0x12
    assert_eq!(buffer[0], 0x78);
    assert_eq!(buffer[1], 0x56);
    assert_eq!(buffer[2], 0x34);
    assert_eq!(buffer[3], 0x12);
}

#[test]
fn test_decode_error_display_buffer_too_short() {
    // Test DecodeError::BufferTooShort display formatting
    let error = DecodeError::BufferTooShort {
        expected: 20,
        actual: 5,
    };
    
    let error_string = error.to_string();
    assert!(error_string.contains("Buffer too short"));
    assert!(error_string.contains("20"));
    assert!(error_string.contains("5"));
}

#[test]
fn test_decode_error_display_invalid_data() {
    // Test DecodeError::InvalidData display formatting
    let error = DecodeError::InvalidData("test error message".to_string());
    
    let error_string = error.to_string();
    assert!(error_string.contains("Invalid data"));
    assert!(error_string.contains("test error message"));
}
