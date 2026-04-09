// Buffer Size Correctness Property Tests
// **Property 5: Buffer Size Correctness**
// **Validates: Requirement 3.2**
//
// These tests verify that for any collection of N entities, BinaryEncoder.encode()
// produces a buffer of exactly (8 + N * 20) bytes.

use protocol::{BinaryEncoder, EntityState, ENTITY_SIZE, HEADER_SIZE};
use proptest::prelude::*;

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

/// Strategy for generating a collection of entities (0 to 1000 entities)
/// Using a wider range to test various sizes including the 1000 entity target
fn entities_strategy() -> impl Strategy<Value = Vec<EntityState>> {
    prop::collection::vec(entity_strategy(), 0..=1000)
}

// Property-based tests using proptest

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Buffer size correctness for any entity collection
    ///
    /// For any collection of N entities, BinaryEncoder.encode() SHALL produce
    /// a buffer of exactly (8 + N * 20) bytes.
    ///
    /// This validates that:
    /// - HEADER is always 8 bytes
    /// - Each ENTITY is always 20 bytes
    /// - No padding or extra bytes are added
    /// - Buffer size formula is: 8 + (entity_count * 20)
    #[test]
    fn prop_buffer_size_matches_formula(entities in entities_strategy()) {
        let buffer = BinaryEncoder::encode(&entities);
        let expected_size = HEADER_SIZE + entities.len() * ENTITY_SIZE;
        
        prop_assert_eq!(
            buffer.len(),
            expected_size,
            "Buffer size mismatch for {} entities: expected {} bytes, got {} bytes",
            entities.len(),
            expected_size,
            buffer.len()
        );
    }

    /// Property: Buffer size correctness with explicit formula
    ///
    /// Verify the exact formula: buffer_size = 8 + (N * 20)
    #[test]
    fn prop_buffer_size_explicit_formula(entities in entities_strategy()) {
        let buffer = BinaryEncoder::encode(&entities);
        let n = entities.len();
        let expected_size = 8 + (n * 20);
        
        prop_assert_eq!(
            buffer.len(),
            expected_size,
            "Buffer size doesn't match formula 8 + (N * 20) for N={}: expected {}, got {}",
            n,
            expected_size,
            buffer.len()
        );
    }

    /// Property: Buffer size scales linearly with entity count
    ///
    /// Adding one entity should increase buffer size by exactly 20 bytes.
    #[test]
    fn prop_buffer_size_scales_linearly(
        entities in prop::collection::vec(entity_strategy(), 1..=100)
    ) {
        let buffer_n = BinaryEncoder::encode(&entities);
        
        // Add one more entity
        let mut entities_plus_one = entities.clone();
        entities_plus_one.push(EntityState::new(9999, 0.0, 0.0, 0.0, 0));
        let buffer_n_plus_1 = BinaryEncoder::encode(&entities_plus_one);
        
        prop_assert_eq!(
            buffer_n_plus_1.len(),
            buffer_n.len() + ENTITY_SIZE,
            "Adding one entity should increase buffer size by exactly {} bytes",
            ENTITY_SIZE
        );
    }

    /// Property: Header size is constant regardless of entity count
    ///
    /// The first 8 bytes are always the header, regardless of entity count.
    #[test]
    fn prop_header_size_constant(entities in entities_strategy()) {
        let buffer = BinaryEncoder::encode(&entities);
        
        // Buffer must be at least HEADER_SIZE bytes
        prop_assert!(
            buffer.len() >= HEADER_SIZE,
            "Buffer must be at least {} bytes (header size), got {}",
            HEADER_SIZE,
            buffer.len()
        );
        
        // Verify header contains entity count
        let entity_count = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        prop_assert_eq!(
            entity_count as usize,
            entities.len(),
            "Header entity_count field should match actual entity count"
        );
    }
}

// Standard unit tests for specific edge cases

#[test]
fn test_buffer_size_empty_collection() {
    let entities = vec![];
    let buffer = BinaryEncoder::encode(&entities);
    
    // Empty collection: only header (8 bytes)
    assert_eq!(buffer.len(), 8);
    assert_eq!(buffer.len(), HEADER_SIZE);
}

#[test]
fn test_buffer_size_single_entity() {
    let entities = vec![EntityState::new(1, 10.0, 20.0, 1.5, 0)];
    let buffer = BinaryEncoder::encode(&entities);
    
    // 1 entity: header (8 bytes) + 1 entity (20 bytes) = 28 bytes
    assert_eq!(buffer.len(), 28);
    assert_eq!(buffer.len(), HEADER_SIZE + ENTITY_SIZE);
    assert_eq!(buffer.len(), 8 + (1 * 20));
}

#[test]
fn test_buffer_size_ten_entities() {
    let entities: Vec<EntityState> = (0..10)
        .map(|i| EntityState::new(i, i as f32, i as f32, 0.0, 0))
        .collect();
    let buffer = BinaryEncoder::encode(&entities);
    
    // 10 entities: header (8 bytes) + 10 entities (200 bytes) = 208 bytes
    assert_eq!(buffer.len(), 208);
    assert_eq!(buffer.len(), HEADER_SIZE + 10 * ENTITY_SIZE);
    assert_eq!(buffer.len(), 8 + (10 * 20));
}

#[test]
fn test_buffer_size_hundred_entities() {
    let entities: Vec<EntityState> = (0..100)
        .map(|i| EntityState::new(i, i as f32, i as f32, 0.0, 0))
        .collect();
    let buffer = BinaryEncoder::encode(&entities);
    
    // 100 entities: header (8 bytes) + 100 entities (2000 bytes) = 2008 bytes
    assert_eq!(buffer.len(), 2008);
    assert_eq!(buffer.len(), HEADER_SIZE + 100 * ENTITY_SIZE);
    assert_eq!(buffer.len(), 8 + (100 * 20));
}

#[test]
fn test_buffer_size_thousand_entities() {
    let entities: Vec<EntityState> = (0..1000)
        .map(|i| EntityState::new(i, i as f32, i as f32, 0.0, 0))
        .collect();
    let buffer = BinaryEncoder::encode(&entities);
    
    // 1000 entities: header (8 bytes) + 1000 entities (20000 bytes) = 20008 bytes
    assert_eq!(buffer.len(), 20008);
    assert_eq!(buffer.len(), HEADER_SIZE + 1000 * ENTITY_SIZE);
    assert_eq!(buffer.len(), 8 + (1000 * 20));
}

#[test]
fn test_buffer_size_formula_consistency() {
    // Test various sizes to ensure formula holds
    for n in [0, 1, 5, 10, 50, 100, 500, 1000] {
        let entities: Vec<EntityState> = (0..n)
            .map(|i| EntityState::new(i as u32, i as f32, i as f32, 0.0, 0))
            .collect();
        let buffer = BinaryEncoder::encode(&entities);
        
        let expected_size = 8 + (n * 20);
        assert_eq!(
            buffer.len(),
            expected_size,
            "Buffer size mismatch for {} entities",
            n
        );
    }
}

#[test]
fn test_buffer_size_with_varying_data() {
    // Test that buffer size is independent of entity data values
    let test_cases = vec![
        vec![EntityState::new(0, 0.0, 0.0, 0.0, 0)],
        vec![EntityState::new(u32::MAX, f32::MAX, f32::MIN, std::f32::consts::PI, u32::MAX)],
        vec![EntityState::new(12345, -999.99, 888.88, -3.14159, 42)],
    ];
    
    for entities in test_cases {
        let buffer = BinaryEncoder::encode(&entities);
        assert_eq!(
            buffer.len(),
            8 + (entities.len() * 20),
            "Buffer size should be independent of entity data values"
        );
    }
}
