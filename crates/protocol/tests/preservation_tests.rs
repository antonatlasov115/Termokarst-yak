// Preservation Property Tests for Protocol Crate
// **Validates: Requirements 3.3**
//
// These tests capture baseline behavior on UNFIXED code.
// They should PASS on unfixed code to establish what behavior needs to be preserved.

use protocol::{BinaryEncoder, EntityState, ENTITY_SIZE, HEADER_SIZE};

#[test]
fn test_encode_preserves_format() {
    // Property: encode always produces buffer with correct size
    let test_cases = vec![
        vec![],
        vec![EntityState::new(1, 0.0, 0.0, 0.0, 0)],
        vec![
            EntityState::new(1, 10.0, 20.0, 0.5, 0),
            EntityState::new(2, 30.0, 40.0, 1.0, 1),
        ],
        vec![
            EntityState::new(1, 100.0, 200.0, 1.5, 0),
            EntityState::new(2, 150.0, 250.0, 2.0, 1),
            EntityState::new(3, 200.0, 300.0, 3.14, 2),
        ],
    ];

    for entities in test_cases {
        let buffer = BinaryEncoder::encode(&entities);
        let expected_size = HEADER_SIZE + entities.len() * ENTITY_SIZE;
        assert_eq!(
            buffer.len(),
            expected_size,
            "Buffer size mismatch for {} entities",
            entities.len()
        );

        // Verify header contains correct entity count
        let count = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        assert_eq!(
            count,
            entities.len() as u32,
            "Entity count mismatch in header"
        );
    }
}

#[test]
fn test_encode_f32_preserves_format() {
    // Property: encode_f32 always produces buffer with 5 floats per entity
    let test_cases = vec![
        vec![],
        vec![EntityState::new(1, 0.0, 0.0, 0.0, 0)],
        vec![
            EntityState::new(1, 10.0, 20.0, 0.5, 0),
            EntityState::new(2, 30.0, 40.0, 1.0, 1),
        ],
        vec![
            EntityState::new(1, 100.0, 200.0, 1.5, 0),
            EntityState::new(2, 150.0, 250.0, 2.0, 1),
            EntityState::new(3, 200.0, 300.0, 3.14, 2),
        ],
    ];

    for entities in test_cases {
        let buffer = BinaryEncoder::encode_f32(&entities);
        let expected_size = entities.len() * 5; // 5 floats per entity
        assert_eq!(
            buffer.len(),
            expected_size,
            "Buffer size mismatch for {} entities",
            entities.len()
        );

        // Verify each entity's data is in correct order
        for (i, entity) in entities.iter().enumerate() {
            let offset = i * 5;
            assert_eq!(buffer[offset], entity.id as f32, "ID mismatch at index {}", i);
            assert_eq!(buffer[offset + 1], entity.x, "X mismatch at index {}", i);
            assert_eq!(buffer[offset + 2], entity.y, "Y mismatch at index {}", i);
            assert_eq!(
                buffer[offset + 3],
                entity.rotation,
                "Rotation mismatch at index {}",
                i
            );
            assert_eq!(
                buffer[offset + 4],
                entity.costume_id as f32,
                "Costume ID mismatch at index {}",
                i
            );
        }
    }
}

#[test]
fn test_encode_preserves_byte_order() {
    // Property: encode uses little-endian byte order consistently
    let entity = EntityState::new(0x12345678, 1.0, 2.0, 3.0, 0xABCDEF01);
    let buffer = BinaryEncoder::encode(&[entity]);

    // Check entity count (should be 1)
    let count = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    assert_eq!(count, 1);

    // Check entity ID is encoded in little-endian
    let id = u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);
    assert_eq!(id, 0x12345678);

    // Check costume_id is encoded in little-endian
    let costume_offset = 8 + 4 + 4 + 4 + 4; // header + id + x + y + rotation
    let costume_id = u32::from_le_bytes([
        buffer[costume_offset],
        buffer[costume_offset + 1],
        buffer[costume_offset + 2],
        buffer[costume_offset + 3],
    ]);
    assert_eq!(costume_id, 0xABCDEF01);
}

#[test]
fn test_encode_preserves_float_precision() {
    // Property: encode preserves float values when round-tripped
    let test_floats = vec![
        0.0,
        1.0,
        -1.0,
        3.14159,
        -3.14159,
        100.5,
        0.000001,
        1000000.0,
    ];

    for &x in &test_floats {
        for &y in &test_floats {
            for &rotation in &test_floats {
                let entity = EntityState::new(1, x, y, rotation, 0);
                let buffer = BinaryEncoder::encode(&[entity]);

                // Extract floats from buffer
                let x_offset = 8 + 4;
                let y_offset = x_offset + 4;
                let rot_offset = y_offset + 4;

                let x_decoded = f32::from_le_bytes([
                    buffer[x_offset],
                    buffer[x_offset + 1],
                    buffer[x_offset + 2],
                    buffer[x_offset + 3],
                ]);
                let y_decoded = f32::from_le_bytes([
                    buffer[y_offset],
                    buffer[y_offset + 1],
                    buffer[y_offset + 2],
                    buffer[y_offset + 3],
                ]);
                let rot_decoded = f32::from_le_bytes([
                    buffer[rot_offset],
                    buffer[rot_offset + 1],
                    buffer[rot_offset + 2],
                    buffer[rot_offset + 3],
                ]);

                assert_eq!(x_decoded, x, "X value not preserved");
                assert_eq!(y_decoded, y, "Y value not preserved");
                assert_eq!(rot_decoded, rotation, "Rotation value not preserved");
            }
        }
    }
}
