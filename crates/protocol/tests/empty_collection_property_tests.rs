// Empty Collection Handling Property Tests
// **Property 9: Empty Collection Handling**
// **Validates: Requirements 4.3, 4.5**
//
// These tests verify that empty entity collections (entity_count = 0) are handled
// correctly throughout the encode-decode pipeline. The tests ensure:
// 1. Encoding produces exactly 8 bytes (header only)
// 2. Decoding in TypeScript produces an empty entities array
// 3. No errors are thrown during the round-trip
// 4. The round-trip preserves the empty state

use protocol::{BinaryEncoder, EntityState, HEADER_SIZE};
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
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
fn find_cli_script() -> Result<std::path::PathBuf, String> {
    // Try to find the CLI script relative to the workspace root
    let possible_paths = vec![
        "src/utils/protocol/__tests__/roundtrip-cli.mjs",
        "../../../src/utils/protocol/__tests__/roundtrip-cli.mjs",
        "../../src/utils/protocol/__tests__/roundtrip-cli.mjs",
    ];

    for path in possible_paths {
        let full_path = std::path::PathBuf::from(path);
        if full_path.exists() {
            return Ok(full_path);
        }
    }

    Err("Could not find roundtrip-cli.mjs file. Make sure it exists in src/utils/protocol/__tests__/".to_string())
}

// Property-based tests using proptest

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Empty collection encoding produces exactly 8 bytes
    ///
    /// For an empty entity collection (entity_count = 0), BinaryEncoder.encode()
    /// SHALL produce a buffer of exactly 8 bytes (header only, no entity data).
    ///
    /// This validates that:
    /// - Empty collections are handled correctly
    /// - No entity data is written when entity_count = 0
    /// - Buffer contains only the header (8 bytes)
    #[test]
    fn prop_empty_collection_buffer_size(_dummy in 0u32..1) {
        let entities: Vec<EntityState> = vec![];
        let buffer = BinaryEncoder::encode(&entities);
        
        prop_assert_eq!(
            buffer.len(),
            HEADER_SIZE,
            "Empty collection should produce exactly {} bytes (header only), got {}",
            HEADER_SIZE,
            buffer.len()
        );
        
        prop_assert_eq!(
            buffer.len(),
            8,
            "Empty collection should produce exactly 8 bytes, got {}",
            buffer.len()
        );
    }

    /// Property: Empty collection round-trip preserves empty state
    ///
    /// For an empty entity collection, encoding in Rust then decoding in TypeScript
    /// SHALL produce an empty entities array without errors.
    ///
    /// This validates that:
    /// - Empty collections can be decoded successfully
    /// - Decoded entities array is empty
    /// - No errors are thrown during decoding
    #[test]
    fn prop_empty_collection_roundtrip(_dummy in 0u32..1) {
        let entities: Vec<EntityState> = vec![];
        verify_roundtrip(&entities).unwrap();
    }

    /// Property: Empty collection header contains zero entity_count
    ///
    /// For an empty entity collection, the header SHALL contain entity_count = 0.
    #[test]
    fn prop_empty_collection_header_entity_count(_dummy in 0u32..1) {
        let entities: Vec<EntityState> = vec![];
        let buffer = BinaryEncoder::encode(&entities);
        
        // Extract entity_count from header (first 4 bytes)
        let entity_count = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        
        prop_assert_eq!(
            entity_count,
            0,
            "Empty collection header should contain entity_count = 0, got {}",
            entity_count
        );
    }
}

// Standard unit tests for specific edge cases

#[test]
fn test_empty_collection_buffer_size() {
    let entities: Vec<EntityState> = vec![];
    let buffer = BinaryEncoder::encode(&entities);
    
    // Empty collection should produce exactly 8 bytes (header only)
    assert_eq!(buffer.len(), 8);
    assert_eq!(buffer.len(), HEADER_SIZE);
}

#[test]
fn test_empty_collection_header_content() {
    let entities: Vec<EntityState> = vec![];
    let buffer = BinaryEncoder::encode(&entities);
    
    // Verify header contains entity_count = 0
    let entity_count = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    assert_eq!(entity_count, 0);
    
    // Verify timestamp field exists (4 bytes at offset 4)
    let timestamp = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
    // Timestamp should be 0 (current implementation)
    assert_eq!(timestamp, 0);
}

#[test]
fn test_empty_collection_roundtrip() {
    let entities: Vec<EntityState> = vec![];
    verify_roundtrip(&entities).expect("Round-trip failed for empty collection");
}

#[test]
fn test_empty_collection_no_entity_data() {
    let entities: Vec<EntityState> = vec![];
    let buffer = BinaryEncoder::encode(&entities);
    
    // Buffer should contain only header, no entity data
    assert_eq!(buffer.len(), 8);
    
    // Verify no bytes beyond the header
    assert!(buffer.len() <= HEADER_SIZE);
}

#[test]
fn test_empty_collection_multiple_times() {
    // Verify encoding empty collection multiple times produces consistent results
    for _ in 0..10 {
        let entities: Vec<EntityState> = vec![];
        let buffer = BinaryEncoder::encode(&entities);
        
        assert_eq!(buffer.len(), 8);
        
        let entity_count = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        assert_eq!(entity_count, 0);
    }
}

#[test]
fn test_empty_vs_single_entity_size_difference() {
    // Verify the size difference between empty and single entity
    let empty_entities: Vec<EntityState> = vec![];
    let empty_buffer = BinaryEncoder::encode(&empty_entities);
    
    let single_entity = vec![EntityState::new(1, 0.0, 0.0, 0.0, 0)];
    let single_buffer = BinaryEncoder::encode(&single_entity);
    
    // Single entity buffer should be exactly 20 bytes larger
    assert_eq!(single_buffer.len(), empty_buffer.len() + 20);
    assert_eq!(empty_buffer.len(), 8);
    assert_eq!(single_buffer.len(), 28);
}
