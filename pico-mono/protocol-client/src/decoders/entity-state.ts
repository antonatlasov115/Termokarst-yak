import type { BinaryDecoder } from '../types';

/**
 * Entity state representation in the protocol.
 * 
 * Represents a single entity's state with position, rotation, and visual properties.
 * This matches the Rust EntityState structure (20 bytes total).
 * 
 * Binary format (little-endian):
 * - id: u32 (4 bytes) - Entity identifier
 * - x: f32 (4 bytes) - X position
 * - y: f32 (4 bytes) - Y position
 * - rotation: f32 (4 bytes) - Rotation in radians
 * - costume_id: u32 (4 bytes) - Visual costume identifier
 * 
 * **Validates: Requirements 5.4**
 */
export interface EntityState {
  /** Entity unique identifier */
  id: number;
  
  /** X coordinate position */
  x: number;
  
  /** Y coordinate position */
  y: number;
  
  /** Rotation angle in radians */
  rotation: number;
  
  /** Costume/sprite identifier */
  costume_id: number;
}

/**
 * Binary decoder for EntityState messages.
 * 
 * Decodes 20-byte binary buffers into EntityState objects.
 * All numeric values are encoded in little-endian format.
 * 
 * **Validates: Requirements 5.4**
 * 
 * @example
 * ```typescript
 * const decoder = new EntityStateDecoder();
 * const buffer = new Uint8Array(20); // ... filled with data
 * const entity = decoder.decode(buffer);
 * console.log(entity.id, entity.x, entity.y);
 * ```
 */
export class EntityStateDecoder implements BinaryDecoder<EntityState> {
  /**
   * Size in bytes of a single EntityState (20 bytes).
   * 
   * Layout:
   * - id: 4 bytes (u32)
   * - x: 4 bytes (f32)
   * - y: 4 bytes (f32)
   * - rotation: 4 bytes (f32)
   * - costume_id: 4 bytes (u32)
   */
  readonly byteSize = 20;

  /**
   * Decode a 20-byte buffer into an EntityState object.
   * 
   * @param buffer - Binary data containing entity state (must be at least 20 bytes)
   * @returns Decoded EntityState object
   * @throws Error if buffer is too short
   * 
   * @example
   * ```typescript
   * const decoder = new EntityStateDecoder();
   * const buffer = new Uint8Array([
   *   1, 0, 0, 0,           // id = 1
   *   0, 0, 200, 66,        // x = 100.0
   *   0, 0, 72, 67,         // y = 200.0
   *   0, 0, 192, 63,        // rotation = 1.5
   *   0, 0, 0, 0            // costume_id = 0
   * ]);
   * const entity = decoder.decode(buffer);
   * ```
   */
  decode(buffer: Uint8Array): EntityState {
    if (buffer.length < this.byteSize) {
      throw new Error(
        `Buffer too short for EntityState: expected ${this.byteSize} bytes, got ${buffer.length}`
      );
    }

    const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);

    return {
      id: view.getUint32(0, true),           // offset 0, little-endian
      x: view.getFloat32(4, true),           // offset 4, little-endian
      y: view.getFloat32(8, true),           // offset 8, little-endian
      rotation: view.getFloat32(12, true),   // offset 12, little-endian
      costume_id: view.getUint32(16, true),  // offset 16, little-endian
    };
  }
}
