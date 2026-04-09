/// Unit tests for ProtocolCodec
///
/// **Validates: Requirements 8.2**
///
/// These tests verify the ProtocolCodec functionality including:
/// - encode() with version 1
/// - encode_legacy() without version (backward compatibility)
/// - decode() with correct version
/// - decode() with incorrect version (should error)
/// - roundtrip through codec
use protocol::{ProtocolCodec, EntityState, DecodeError, BinarySerialize};
use proptest::prelude::*;

#[test]
fn test_encode_with_version_1() {
    let codec = ProtocolCodec::<EntityState>::new(1);
    let entities = vec![
        EntityState::new(1, 10.0, 20.0, 0.5, 0),
        EntityState::new(2, 30.0, 40.0, 1.0, 1),
    ];

    let buffer = codec.encode(&entities);

    // Verify version byte is present and correct
    assert_eq!(buffer[0], 1, "First byte should be version 1");

    // Verify count
    let count = u32::from_le_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]);
    assert_eq!(count, 2, "Count should be 2 entities");

    // Verify timestamp is present (4 bytes after count)
    let timestamp = u32::from_le_bytes([buffer[5], buffer[6], buffer[7], buffer[8]]);
    assert!(timestamp > 0, "Timestamp should be non-zero");

    // Verify total size: 1 (version) + 4 (count) + 4 (timestamp) + 2 * 20 (entities)
    assert_eq!(buffer.len(), 1 + 4 + 4 + 40, "Buffer size should be correct");
}

#[test]
fn test_encode_legacy_without_version() {
    let codec = ProtocolCodec::<EntityState>::new(1);
    let entities = vec![
        EntityState::new(1, 10.0, 20.0, 0.5, 0),
        EntityState::new(2, 30.0, 40.0, 1.0, 1),
    ];

    let buffer = codec.encode_legacy(&entities);

    // Verify NO version byte - buffer starts with count
    let count = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    assert_eq!(count, 2, "First 4 bytes should be count (no version byte)");

    // Verify timestamp
    let timestamp = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
    assert_eq!(timestamp, 0, "Legacy format uses 0 timestamp");

    // Verify total size: 4 (count) + 4 (timestamp) + 2 * 20 (entities)
    assert_eq!(buffer.len(), 4 + 4 + 40, "Buffer size should be correct (no version byte)");

    // Verify first entity starts at offset 8 (not 9)
    let first_entity_id = u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);
    assert_eq!(first_entity_id, 1, "First entity should start at offset 8");
}

#[test]
fn test_decode_with_correct_version() {
    let codec = ProtocolCodec::<EntityState>::new(1);
    let entities = vec![
        EntityState::new(42, 100.5, 200.75, 1.57, 3),
        EntityState::new(99, 50.0, 75.0, 2.5, 7),
    ];

    let buffer = codec.encode(&entities);
    let decoded = codec.decode(&buffer).expect("Decode should succeed with correct version");

    // Verify version
    assert_eq!(decoded.version, 1, "Decoded version should be 1");

    // Verify timestamp is present
    assert!(decoded.timestamp > 0, "Timestamp should be non-zero");

    // Verify entity count
    assert_eq!(decoded.items.len(), 2, "Should decode 2 entities");

    // Verify first entity
    assert_eq!(decoded.items[0].id, 42);
    assert!((decoded.items[0].x - 100.5).abs() < 0.001);
    assert!((decoded.items[0].y - 200.75).abs() < 0.001);
    assert!((decoded.items[0].rotation - 1.57).abs() < 0.001);
    assert_eq!(decoded.items[0].costume_id, 3);

    // Verify second entity
    assert_eq!(decoded.items[1].id, 99);
    assert!((decoded.items[1].x - 50.0).abs() < 0.001);
    assert!((decoded.items[1].y - 75.0).abs() < 0.001);
    assert!((decoded.items[1].rotation - 2.5).abs() < 0.001);
    assert_eq!(decoded.items[1].costume_id, 7);
}

#[test]
fn test_decode_with_incorrect_version_should_error() {
    let codec_v1 = ProtocolCodec::<EntityState>::new(1);
    let codec_v2 = ProtocolCodec::<EntityState>::new(2);

    let entities = vec![EntityState::new(1, 10.0, 20.0, 0.5, 0)];

    // Encode with version 1
    let buffer = codec_v1.encode(&entities);

    // Try to decode with version 2 codec - should fail
    let result = codec_v2.decode(&buffer);

    assert!(result.is_err(), "Decode should fail with version mismatch");

    match result {
        Err(DecodeError::InvalidData(msg)) => {
            assert!(msg.contains("Version mismatch"), "Error should mention version mismatch");
            assert!(msg.contains("expected 2"), "Error should mention expected version 2");
            assert!(msg.contains("got 1"), "Error should mention got version 1");
        }
        _ => panic!("Expected InvalidData error with version mismatch message"),
    }
}

#[test]
fn test_roundtrip_through_codec() {
    let codec = ProtocolCodec::<EntityState>::new(1);

    let original_entities = vec![
        EntityState::new(1, 10.0, 20.0, 0.5, 0),
        EntityState::new(2, 30.0, 40.0, 1.0, 1),
        EntityState::new(3, 50.0, 60.0, 1.5, 2),
        EntityState::new(4, 70.0, 80.0, 2.0, 3),
    ];

    // Encode
    let buffer = codec.encode(&original_entities);

    // Decode
    let decoded = codec.decode(&buffer).expect("Roundtrip decode should succeed");

    // Verify all entities match
    assert_eq!(decoded.items.len(), original_entities.len(), "Entity count should match");

    for (original, decoded_entity) in original_entities.iter().zip(decoded.items.iter()) {
        assert_eq!(decoded_entity.id, original.id, "Entity ID should match");
        assert!(
            (decoded_entity.x - original.x).abs() < 0.001,
            "Entity X coordinate should match"
        );
        assert!(
            (decoded_entity.y - original.y).abs() < 0.001,
            "Entity Y coordinate should match"
        );
        assert!(
            (decoded_entity.rotation - original.rotation).abs() < 0.001,
            "Entity rotation should match"
        );
        assert_eq!(
            decoded_entity.costume_id, original.costume_id,
            "Entity costume_id should match"
        );
    }
}

#[test]
fn test_roundtrip_empty_collection() {
    let codec = ProtocolCodec::<EntityState>::new(1);
    let entities: Vec<EntityState> = vec![];

    let buffer = codec.encode(&entities);
    let decoded = codec.decode(&buffer).expect("Should decode empty collection");

    assert_eq!(decoded.items.len(), 0, "Should decode to empty collection");
    assert_eq!(decoded.version, 1, "Version should be preserved");
}

#[test]
fn test_roundtrip_single_entity() {
    let codec = ProtocolCodec::<EntityState>::new(1);
    let entities = vec![EntityState::new(123, 456.78, 901.23, 3.14159, 42)];

    let buffer = codec.encode(&entities);
    let decoded = codec.decode(&buffer).expect("Should decode single entity");

    assert_eq!(decoded.items.len(), 1, "Should decode 1 entity");
    assert_eq!(decoded.items[0].id, 123);
    assert!((decoded.items[0].x - 456.78).abs() < 0.001);
    assert!((decoded.items[0].y - 901.23).abs() < 0.001);
    assert!((decoded.items[0].rotation - 3.14159).abs() < 0.001);
    assert_eq!(decoded.items[0].costume_id, 42);
}

#[test]
fn test_encode_legacy_backward_compatibility() {
    let codec = ProtocolCodec::<EntityState>::new(1);
    let entities = vec![EntityState::new(1, 100.0, 200.0, 1.5, 0)];

    let legacy_buffer = codec.encode_legacy(&entities);

    // Legacy format should be compatible with old decoder expectations
    // Format: [COUNT:u32][TIMESTAMP:u32][ENTITY_DATA...]
    assert_eq!(legacy_buffer.len(), 8 + 20, "Legacy buffer should be 28 bytes");

    // Verify count is at the start
    let count = u32::from_le_bytes([
        legacy_buffer[0],
        legacy_buffer[1],
        legacy_buffer[2],
        legacy_buffer[3],
    ]);
    assert_eq!(count, 1, "Count should be first field in legacy format");

    // Verify entity data starts at offset 8
    let entity_id = u32::from_le_bytes([
        legacy_buffer[8],
        legacy_buffer[9],
        legacy_buffer[10],
        legacy_buffer[11],
    ]);
    assert_eq!(entity_id, 1, "Entity data should start at offset 8");
}

#[test]
fn test_different_codec_versions() {
    let codec_v1 = ProtocolCodec::<EntityState>::new(1);
    let codec_v5 = ProtocolCodec::<EntityState>::new(5);
    let codec_v255 = ProtocolCodec::<EntityState>::new(255);

    let entities = vec![EntityState::new(1, 10.0, 20.0, 0.5, 0)];

    // Encode with different versions
    let buffer_v1 = codec_v1.encode(&entities);
    let buffer_v5 = codec_v5.encode(&entities);
    let buffer_v255 = codec_v255.encode(&entities);

    // Verify version bytes
    assert_eq!(buffer_v1[0], 1, "Version 1 buffer should have version byte 1");
    assert_eq!(buffer_v5[0], 5, "Version 5 buffer should have version byte 5");
    assert_eq!(buffer_v255[0], 255, "Version 255 buffer should have version byte 255");

    // Each codec can decode its own version
    assert!(codec_v1.decode(&buffer_v1).is_ok(), "Codec v1 should decode v1 buffer");
    assert!(codec_v5.decode(&buffer_v5).is_ok(), "Codec v5 should decode v5 buffer");
    assert!(codec_v255.decode(&buffer_v255).is_ok(), "Codec v255 should decode v255 buffer");

    // But not other versions
    assert!(codec_v1.decode(&buffer_v5).is_err(), "Codec v1 should reject v5 buffer");
    assert!(codec_v5.decode(&buffer_v1).is_err(), "Codec v5 should reject v1 buffer");
    assert!(codec_v1.decode(&buffer_v255).is_err(), "Codec v1 should reject v255 buffer");
}

// ============================================================================
// Property-Based Tests for Generic Codec
// ============================================================================
// **Validates: Requirements 8.2**
//
// These property tests verify that the generic ProtocolCodec works correctly
// for arbitrary collections of EntityState objects.

/// Strategy for generating valid entity IDs (0 to 10,000)
fn entity_id_strategy() -> impl Strategy<Value = u32> {
    0u32..=10_000
}

/// Strategy for generating valid positions (-1000.0 to 1000.0)
fn position_strategy() -> impl Strategy<Value = f32> {
    -1000.0f32..=1000.0
}

/// Strategy for generating valid rotations (-π to π)
fn rotation_strategy() -> impl Strategy<Value = f32> {
    -std::f32::consts::PI..=std::f32::consts::PI
}

/// Strategy for generating valid costume IDs (0 to 100)
fn costume_id_strategy() -> impl Strategy<Value = u32> {
    0u32..=100
}

/// Strategy for generating a single EntityState
fn entity_strategy() -> impl Strategy<Value = EntityState> {
    (
        entity_id_strategy(),
        position_strategy(),
        position_strategy(),
        rotation_strategy(),
        costume_id_strategy(),
    )
        .prop_map(|(id, x, y, rotation, costume_id)| EntityState::new(id, x, y, rotation, costume_id))
}

/// Strategy for generating a collection of entities (0 to 100 entities)
fn entities_strategy() -> impl Strategy<Value = Vec<EntityState>> {
    prop::collection::vec(entity_strategy(), 0..=100)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Roundtrip для любой коллекции EntityState
    ///
    /// **Validates: Requirements 8.2**
    ///
    /// For any collection of EntityState objects, encoding with ProtocolCodec
    /// and then decoding SHALL produce equivalent entity data where all fields
    /// match within floating-point precision (epsilon = 0.001).
    #[test]
    fn prop_codec_roundtrip_preserves_data(entities in entities_strategy()) {
        let codec = ProtocolCodec::<EntityState>::new(1);
        
        // Encode
        let buffer = codec.encode(&entities);
        
        // Decode
        let decoded = codec.decode(&buffer).expect("Decode should succeed");
        
        // Verify all entities match
        prop_assert_eq!(decoded.items.len(), entities.len(), "Entity count should match");
        
        for (original, decoded_entity) in entities.iter().zip(decoded.items.iter()) {
            prop_assert_eq!(decoded_entity.id, original.id, "Entity ID should match");
            prop_assert!(
                (decoded_entity.x - original.x).abs() < 0.001,
                "Entity X coordinate should match (original: {}, decoded: {})",
                original.x,
                decoded_entity.x
            );
            prop_assert!(
                (decoded_entity.y - original.y).abs() < 0.001,
                "Entity Y coordinate should match (original: {}, decoded: {})",
                original.y,
                decoded_entity.y
            );
            prop_assert!(
                (decoded_entity.rotation - original.rotation).abs() < 0.001,
                "Entity rotation should match (original: {}, decoded: {})",
                original.rotation,
                decoded_entity.rotation
            );
            prop_assert_eq!(
                decoded_entity.costume_id,
                original.costume_id,
                "Entity costume_id should match"
            );
        }
    }

    /// Property: Размер буфера = 9 + sum(item.byte_size())
    ///
    /// **Validates: Requirements 8.2**
    ///
    /// For any collection of N entities, ProtocolCodec.encode() SHALL produce
    /// a buffer of exactly (9 + N * 20) bytes, where:
    /// - 1 byte for version
    /// - 4 bytes for count
    /// - 4 bytes for timestamp
    /// - N * 20 bytes for entity data
    #[test]
    fn prop_codec_buffer_size_formula(entities in entities_strategy()) {
        let codec = ProtocolCodec::<EntityState>::new(1);
        let buffer = codec.encode(&entities);
        
        // Calculate expected size: 1 (version) + 4 (count) + 4 (timestamp) + sum(item.byte_size())
        let expected_size = 9 + entities.iter().map(|e| e.byte_size()).sum::<usize>();
        
        prop_assert_eq!(
            buffer.len(),
            expected_size,
            "Buffer size should match formula: 9 + sum(item.byte_size())"
        );
        
        // Also verify explicit formula: 9 + (N * 20)
        let explicit_size = 9 + (entities.len() * 20);
        prop_assert_eq!(
            buffer.len(),
            explicit_size,
            "Buffer size should match explicit formula: 9 + (N * 20)"
        );
    }

    /// Property: Версия в буфере соответствует версии codec
    ///
    /// **Validates: Requirements 8.2**
    ///
    /// For any codec version V and any collection of entities, the encoded
    /// buffer SHALL have version byte V at position 0.
    #[test]
    fn prop_codec_version_in_buffer(
        version in 0u8..=255u8,
        entities in entities_strategy()
    ) {
        let codec = ProtocolCodec::<EntityState>::new(version);
        let buffer = codec.encode(&entities);
        
        // Verify buffer is not empty
        prop_assert!(!buffer.is_empty(), "Buffer should not be empty");
        
        // Verify version byte at position 0
        prop_assert_eq!(
            buffer[0],
            version,
            "Version byte at position 0 should match codec version"
        );
        
        // Verify decode succeeds with matching version
        let decoded = codec.decode(&buffer).expect("Decode should succeed with matching version");
        prop_assert_eq!(
            decoded.version,
            version,
            "Decoded version should match codec version"
        );
    }

    /// Property: Buffer size scales linearly with entity count
    ///
    /// **Validates: Requirements 8.2**
    ///
    /// Adding one entity should increase buffer size by exactly 20 bytes.
    #[test]
    fn prop_codec_buffer_size_scales_linearly(
        entities in prop::collection::vec(entity_strategy(), 1..=50)
    ) {
        let codec = ProtocolCodec::<EntityState>::new(1);
        
        let buffer_n = codec.encode(&entities);
        
        // Add one more entity
        let mut entities_plus_one = entities.clone();
        entities_plus_one.push(EntityState::new(9999, 0.0, 0.0, 0.0, 0));
        let buffer_n_plus_1 = codec.encode(&entities_plus_one);
        
        prop_assert_eq!(
            buffer_n_plus_1.len(),
            buffer_n.len() + 20,
            "Adding one entity should increase buffer size by exactly 20 bytes"
        );
    }

    /// Property: Version mismatch detection
    ///
    /// **Validates: Requirements 8.2**
    ///
    /// Decoding a buffer with version V using a codec with version V' (where V ≠ V')
    /// SHALL result in a DecodeError::InvalidData error.
    #[test]
    fn prop_codec_version_mismatch_detection(
        version1 in 0u8..=127u8,
        version2 in 128u8..=255u8,
        entities in entities_strategy()
    ) {
        // Ensure versions are different
        prop_assume!(version1 != version2);
        
        let codec_v1 = ProtocolCodec::<EntityState>::new(version1);
        let codec_v2 = ProtocolCodec::<EntityState>::new(version2);
        
        // Encode with version1
        let buffer = codec_v1.encode(&entities);
        
        // Try to decode with version2 - should fail
        let result = codec_v2.decode(&buffer);
        
        prop_assert!(
            result.is_err(),
            "Decode should fail with version mismatch"
        );
        
        match result {
            Err(DecodeError::InvalidData(msg)) => {
                prop_assert!(
                    msg.contains("Version mismatch"),
                    "Error message should mention version mismatch"
                );
            }
            _ => {
                return Err(proptest::test_runner::TestCaseError::fail(
                    "Expected InvalidData error with version mismatch"
                ));
            }
        }
    }

    /// Property: Empty collection handling
    ///
    /// **Validates: Requirements 8.2**
    ///
    /// Encoding and decoding an empty collection SHALL work correctly,
    /// producing a buffer of exactly 9 bytes (header only).
    #[test]
    fn prop_codec_empty_collection(version in 0u8..=255u8) {
        let codec = ProtocolCodec::<EntityState>::new(version);
        let entities: Vec<EntityState> = vec![];
        
        let buffer = codec.encode(&entities);
        
        // Verify buffer size is exactly 9 bytes (header only)
        prop_assert_eq!(buffer.len(), 9, "Empty collection should produce 9-byte buffer");
        
        // Verify version byte
        prop_assert_eq!(buffer[0], version, "Version byte should match");
        
        // Verify count is 0
        let count = u32::from_le_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]);
        prop_assert_eq!(count, 0, "Count should be 0 for empty collection");
        
        // Verify decode succeeds
        let decoded = codec.decode(&buffer).expect("Decode should succeed for empty collection");
        prop_assert_eq!(decoded.items.len(), 0, "Decoded collection should be empty");
        prop_assert_eq!(decoded.version, version, "Decoded version should match");
    }

    /// Property: Timestamp is present and non-zero
    ///
    /// **Validates: Requirements 8.2**
    ///
    /// For any non-empty collection, the encoded buffer SHALL contain a
    /// non-zero timestamp value.
    #[test]
    fn prop_codec_timestamp_present(
        entities in prop::collection::vec(entity_strategy(), 1..=50)
    ) {
        let codec = ProtocolCodec::<EntityState>::new(1);
        let buffer = codec.encode(&entities);
        
        let decoded = codec.decode(&buffer).expect("Decode should succeed");
        
        // Timestamp should be non-zero (current time)
        prop_assert!(
            decoded.timestamp > 0,
            "Timestamp should be non-zero for non-empty collection"
        );
    }

    /// Property: Extreme values handling
    ///
    /// **Validates: Requirements 8.2**
    ///
    /// Encoding and decoding SHALL work correctly for extreme but valid values.
    #[test]
    fn prop_codec_extreme_values(
        id in 0u32..=u32::MAX,
        x in -1_000_000.0f32..=1_000_000.0,
        y in -1_000_000.0f32..=1_000_000.0,
        rotation in -10.0f32..=10.0,
        costume_id in 0u32..=u32::MAX,
    ) {
        let codec = ProtocolCodec::<EntityState>::new(1);
        let entity = EntityState::new(id, x, y, rotation, costume_id);
        let entities = vec![entity];
        
        let buffer = codec.encode(&entities);
        let decoded = codec.decode(&buffer).expect("Decode should succeed with extreme values");
        
        prop_assert_eq!(decoded.items.len(), 1, "Should decode 1 entity");
        prop_assert_eq!(decoded.items[0].id, id, "ID should match");
        prop_assert!(
            (decoded.items[0].x - x).abs() < 0.001,
            "X should match (original: {}, decoded: {})",
            x,
            decoded.items[0].x
        );
        prop_assert!(
            (decoded.items[0].y - y).abs() < 0.001,
            "Y should match (original: {}, decoded: {})",
            y,
            decoded.items[0].y
        );
        prop_assert!(
            (decoded.items[0].rotation - rotation).abs() < 0.001,
            "Rotation should match (original: {}, decoded: {})",
            rotation,
            decoded.items[0].rotation
        );
        prop_assert_eq!(decoded.items[0].costume_id, costume_id, "Costume ID should match");
    }
}
