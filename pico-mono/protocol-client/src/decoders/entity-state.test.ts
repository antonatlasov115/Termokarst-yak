import { describe, it, expect } from 'vitest';
import { EntityStateDecoder } from './entity-state';
import type { EntityState } from './entity-state';

describe('EntityStateDecoder', () => {
  const decoder = new EntityStateDecoder();

  it('should have correct byteSize', () => {
    expect(decoder.byteSize).toBe(20);
  });

  it('should decode a valid entity state', () => {
    // Create a buffer with known values
    const buffer = new Uint8Array(20);
    const view = new DataView(buffer.buffer);
    
    // id = 42
    view.setUint32(0, 42, true);
    // x = 100.5
    view.setFloat32(4, 100.5, true);
    // y = 200.75
    view.setFloat32(8, 200.75, true);
    // rotation = 1.57
    view.setFloat32(12, 1.57, true);
    // costume_id = 3
    view.setUint32(16, 3, true);

    const entity: EntityState = decoder.decode(buffer);

    expect(entity.id).toBe(42);
    expect(entity.x).toBeCloseTo(100.5, 2);
    expect(entity.y).toBeCloseTo(200.75, 2);
    expect(entity.rotation).toBeCloseTo(1.57, 2);
    expect(entity.costume_id).toBe(3);
  });

  it('should decode zero values', () => {
    const buffer = new Uint8Array(20); // All zeros

    const entity = decoder.decode(buffer);

    expect(entity.id).toBe(0);
    expect(entity.x).toBe(0);
    expect(entity.y).toBe(0);
    expect(entity.rotation).toBe(0);
    expect(entity.costume_id).toBe(0);
  });

  it('should decode negative float values', () => {
    const buffer = new Uint8Array(20);
    const view = new DataView(buffer.buffer);
    
    view.setUint32(0, 1, true);
    view.setFloat32(4, -100.0, true);
    view.setFloat32(8, -200.0, true);
    view.setFloat32(12, -3.14, true);
    view.setUint32(16, 0, true);

    const entity = decoder.decode(buffer);

    expect(entity.id).toBe(1);
    expect(entity.x).toBeCloseTo(-100.0, 2);
    expect(entity.y).toBeCloseTo(-200.0, 2);
    expect(entity.rotation).toBeCloseTo(-3.14, 2);
    expect(entity.costume_id).toBe(0);
  });

  it('should decode max u32 values', () => {
    const buffer = new Uint8Array(20);
    const view = new DataView(buffer.buffer);
    
    view.setUint32(0, 0xFFFFFFFF, true);
    view.setFloat32(4, 1000.0, true);
    view.setFloat32(8, 2000.0, true);
    view.setFloat32(12, 6.28, true);
    view.setUint32(16, 0xFFFFFFFF, true);

    const entity = decoder.decode(buffer);

    expect(entity.id).toBe(0xFFFFFFFF);
    expect(entity.x).toBeCloseTo(1000.0, 2);
    expect(entity.y).toBeCloseTo(2000.0, 2);
    expect(entity.rotation).toBeCloseTo(6.28, 2);
    expect(entity.costume_id).toBe(0xFFFFFFFF);
  });

  it('should throw error if buffer is too short', () => {
    const buffer = new Uint8Array(10); // Only 10 bytes

    expect(() => decoder.decode(buffer)).toThrow(
      'Buffer too short for EntityState: expected 20 bytes, got 10'
    );
  });

  it('should throw error if buffer is empty', () => {
    const buffer = new Uint8Array(0);

    expect(() => decoder.decode(buffer)).toThrow(
      'Buffer too short for EntityState: expected 20 bytes, got 0'
    );
  });

  it('should handle buffer with offset correctly', () => {
    // Create a larger buffer with data at an offset
    const largeBuffer = new Uint8Array(30);
    const view = new DataView(largeBuffer.buffer);
    
    // Write entity data starting at offset 5
    view.setUint32(5, 99, true);
    view.setFloat32(9, 50.0, true);
    view.setFloat32(13, 75.0, true);
    view.setFloat32(17, 2.5, true);
    view.setUint32(21, 7, true);

    // Create a slice starting at offset 5
    const buffer = largeBuffer.slice(5, 25);
    const entity = decoder.decode(buffer);

    expect(entity.id).toBe(99);
    expect(entity.x).toBeCloseTo(50.0, 2);
    expect(entity.y).toBeCloseTo(75.0, 2);
    expect(entity.rotation).toBeCloseTo(2.5, 2);
    expect(entity.costume_id).toBe(7);
  });
});
