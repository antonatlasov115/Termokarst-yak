// Round-Trip Property Tests for Binary Protocol
// **Property 1: Round-Trip Data Integrity**
// **Validates: Requirements 4.1, 4.2**
//
// These tests verify that data encoded in Rust using BinaryEncoder.encode()
// can be decoded in TypeScript using BinaryDecoder.decode() with all fields
// matching within floating-point precision (epsilon = 0.0001).

use protocol::{BinaryEncoder, EntityState};
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Entity representation for JSON serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EntityJson {
    id: u32,
    x: f32,
    y: f32,
    rotation: f32,
    costume_id: u32,
}

impl From<&EntityState> for EntityJson {
    fn from(entity: &EntityState) -> Self {
        Self {
            id: entity.id,
            x: entity.x,
            y: entity.y,
            rotation: entity.rotation,
            costume_id: entity.costume_id,
        }
    }
}

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

/// Verify round-trip data integrity by encoding in Rust and decoding in TypeScript
fn verify_roundtrip(entities: &[EntityState]) -> Result<(), String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // Create temporary directory for test files with unique name
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let unique_id = format!("{}_{}", std::process::id(), timestamp);
    let temp_dir = std::env::temp_dir().join(format!("protocol_test_{}", unique_id));
    fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let buffer_path = temp_dir.join("buffer.bin");
    let expected_path = temp_dir.join("expected.json");

    // Encode entities to binary
    let buffer = BinaryEncoder::encode(entities);

    // Write binary buffer to file
    fs::write(&buffer_path, &buffer)
        .map_err(|e| format!("Failed to write buffer file: {}", e))?;

    // Convert entities to JSON format
    let entities_json: Vec<EntityJson> = entities.iter().map(EntityJson::from).collect();
    let json_str =
        serde_json::to_string(&entities_json).map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    // Write expected entities to JSON file
    fs::write(&expected_path, json_str)
        .map_err(|e| format!("Failed to write expected file: {}", e))?;

    // Find the CLI script
    let cli_script = find_cli_script()?;

    // Run TypeScript decoder test via Node.js
    let output = Command::new("node")
        .arg(&cli_script)
        .arg(&buffer_path)
        .arg(&expected_path)
        .output()
        .map_err(|e| format!("Failed to execute Node.js: {}", e))?;

    // Check if the test passed BEFORE cleaning up
    let success = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Clean up temporary files
    let _ = fs::remove_dir_all(&temp_dir);

    if success {
        Ok(())
    } else {
        Err(format!(
            "TypeScript decoder test failed:\nstdout: {}\nstderr: {}",
            stdout, stderr
        ))
    }
}

/// Find the CLI script in the project
fn find_cli_script() -> Result<PathBuf, String> {
    // Try to find the CLI script relative to the workspace root
    let possible_paths = vec![
        "src/utils/protocol/__tests__/roundtrip-cli.mjs",
        "../../../src/utils/protocol/__tests__/roundtrip-cli.mjs",
        "../../src/utils/protocol/__tests__/roundtrip-cli.mjs",
    ];

    for path in possible_paths {
        let full_path = PathBuf::from(path);
        if full_path.exists() {
            return Ok(full_path);
        }
    }

    Err("Could not find roundtrip-cli.mjs file. Make sure it exists in src/utils/protocol/__tests__/".to_string())
}

// Property-based tests using proptest

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    /// Property: Round-trip data integrity for random entity collections
    ///
    /// For any valid collection of EntityState objects, encoding in Rust using
    /// BinaryEncoder.encode() then decoding in TypeScript using BinaryDecoder.decode()
    /// SHALL produce equivalent entity data where all fields match within epsilon = 0.0001.
    #[test]
    fn prop_roundtrip_preserves_all_fields(entities in entities_strategy()) {
        verify_roundtrip(&entities).unwrap();
    }

    /// Property: Round-trip preserves entity count
    ///
    /// The number of decoded entities SHALL equal the number of encoded entities.
    #[test]
    fn prop_roundtrip_preserves_entity_count(entities in entities_strategy()) {
        let buffer = BinaryEncoder::encode(&entities);
        
        // Verify buffer size is correct
        let expected_size = 8 + entities.len() * 20;
        prop_assert_eq!(buffer.len(), expected_size);
        
        // Verify header contains correct entity count
        let entity_count = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        prop_assert_eq!(entity_count as usize, entities.len());
    }

    /// Property: Round-trip handles extreme values
    ///
    /// Encoding and decoding SHALL work correctly for extreme but valid values.
    #[test]
    fn prop_roundtrip_handles_extreme_values(
        id in 0u32..=u32::MAX,
        x in -1_000_000.0f32..=1_000_000.0,
        y in -1_000_000.0f32..=1_000_000.0,
        rotation in -10.0f32..=10.0,
        costume_id in 0u32..=u32::MAX,
    ) {
        let entity = EntityState::new(id, x, y, rotation, costume_id);
        verify_roundtrip(&[entity]).unwrap();
    }
}

// Standard unit tests for specific edge cases

#[test]
fn test_roundtrip_empty_collection() {
    let entities = vec![];
    verify_roundtrip(&entities).expect("Round-trip failed for empty collection");
}

#[test]
fn test_roundtrip_single_entity() {
    let entities = vec![EntityState::new(1, 10.5, 20.5, 1.57, 5)];
    verify_roundtrip(&entities).expect("Round-trip failed for single entity");
}

#[test]
fn test_roundtrip_multiple_entities() {
    let entities = vec![
        EntityState::new(1, 0.0, 0.0, 0.0, 0),
        EntityState::new(2, 100.5, 200.5, 3.14159, 1),
        EntityState::new(3, -50.25, -100.75, -1.57, 2),
    ];
    verify_roundtrip(&entities).expect("Round-trip failed for multiple entities");
}

#[test]
fn test_roundtrip_negative_values() {
    let entities = vec![EntityState::new(1, -10.5, -20.5, -3.14, 0)];
    verify_roundtrip(&entities).expect("Round-trip failed for negative values");
}

#[test]
fn test_roundtrip_zero_values() {
    let entities = vec![EntityState::new(0, 0.0, 0.0, 0.0, 0)];
    verify_roundtrip(&entities).expect("Round-trip failed for zero values");
}

#[test]
fn test_roundtrip_large_collection() {
    let entities: Vec<EntityState> = (0..100)
        .map(|i| {
            EntityState::new(
                i,
                (i as f32) * 10.0,
                (i as f32) * 20.0,
                (i as f32) * 0.1,
                i % 10,
            )
        })
        .collect();
    verify_roundtrip(&entities).expect("Round-trip failed for large collection");
}

#[test]
fn test_roundtrip_special_float_values() {
    let entities = vec![
        EntityState::new(1, 0.0, -0.0, 0.0, 0),
        EntityState::new(2, 0.0001, 0.0001, 0.0001, 1),
        EntityState::new(3, 999999.0, -999999.0, 3.14159265, 2),
    ];
    verify_roundtrip(&entities).expect("Round-trip failed for special float values");
}
