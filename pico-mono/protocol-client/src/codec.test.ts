/**
 * Unit tests for ProtocolCodec
 * 
 * **Validates: Requirements 5.2, 5.3, 6.1**
 * 
 * These tests verify the ProtocolCodec functionality including:
 * - decode() with version checking
 * - decodeLegacy() for backward compatibility
 * - Error handling for invalid buffers
 */

import { describe, it, expect } from 'vitest';
import { ProtocolCodec } from './codec';
import type { BinaryDecoder } from './types';

// Simple test decoder for a mock entity (id: u32, value: f32)
interface TestEntity {
  id: number;
  value: number;
}

class TestEntityDecoder implements BinaryDecoder<TestEntity> {
  readonly byteSize = 8; // 4 bytes for id + 4 bytes for value

  decode(buffer: Uint8Array): TestEntity {
    const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
    return {
      id: view.getUint32(0, true),
      value: view.getFloat32(4, true),
    };
  }
}

describe('ProtocolCodec', () => {
  describe('decode()', () => {
    it('should decode a buffer with correct version', () => {
      const codec = new ProtocolCodec(new TestEntityDecoder(), 1);

      // Create a test buffer: [VERSION:1][COUNT:2][TIMESTAMP:1000][ENTITY1][ENTITY2]
      const buffer = new Uint8Array(9 + 16); // header + 2 entities
      const view = new DataView(buffer.buffer);

      // Header
      view.setUint8(0, 1); // version
      view.setUint32(1, 2, true); // count
      view.setUint32(5, 1000, true); // timestamp

      // Entity 1: id=42, value=3.14
      view.setUint32(9, 42, true);
      view.setFloat32(13, 3.14, true);

      // Entity 2: id=99, value=2.71
      view.setUint32(17, 99, true);
      view.setFloat32(21, 2.71, true);

      const decoded = codec.decode(buffer);

      expect(decoded.version).toBe(1);
      expect(decoded.timestamp).toBe(1000);
      expect(decoded.items).toHaveLength(2);
      expect(decoded.items[0].id).toBe(42);
      expect(decoded.items[0].value).toBeCloseTo(3.14, 2);
      expect(decoded.items[1].id).toBe(99);
      expect(decoded.items[1].value).toBeCloseTo(2.71, 2);
    });

    it('should throw error if buffer is too short', () => {
      const codec = new ProtocolCodec(new TestEntityDecoder(), 1);
      const buffer = new Uint8Array(5); // Only 5 bytes, need at least 9

      expect(() => codec.decode(buffer)).toThrow('Buffer too short');
    });

    it('should throw error on version mismatch', () => {
      const codec = new ProtocolCodec(new TestEntityDecoder(), 2);

      // Create buffer with version 1
      const buffer = new Uint8Array(9);
      const view = new DataView(buffer.buffer);
      view.setUint8(0, 1); // version 1
      view.setUint32(1, 0, true); // count 0
      view.setUint32(5, 0, true); // timestamp 0

      expect(() => codec.decode(buffer)).toThrow('Version mismatch: expected 2, got 1');
    });

    it('should decode empty collection', () => {
      const codec = new ProtocolCodec(new TestEntityDecoder(), 1);

      // Create buffer with 0 entities
      const buffer = new Uint8Array(9);
      const view = new DataView(buffer.buffer);
      view.setUint8(0, 1); // version
      view.setUint32(1, 0, true); // count 0
      view.setUint32(5, 1234, true); // timestamp

      const decoded = codec.decode(buffer);

      expect(decoded.version).toBe(1);
      expect(decoded.timestamp).toBe(1234);
      expect(decoded.items).toHaveLength(0);
    });

    it('should throw error if buffer is too short for items', () => {
      const codec = new ProtocolCodec(new TestEntityDecoder(), 1);

      // Create buffer claiming 2 entities but only has space for 1
      const buffer = new Uint8Array(9 + 8); // header + 1 entity
      const view = new DataView(buffer.buffer);
      view.setUint8(0, 1); // version
      view.setUint32(1, 2, true); // count 2 (but only 1 entity in buffer)
      view.setUint32(5, 0, true); // timestamp

      expect(() => codec.decode(buffer)).toThrow('Buffer too short for item');
    });
  });

  describe('decodeLegacy()', () => {
    it('should decode legacy format without version byte', () => {
      const codec = new ProtocolCodec(new TestEntityDecoder(), 1);

      // Create legacy buffer: [COUNT:2][TIMESTAMP:500][ENTITY1][ENTITY2]
      const buffer = new Uint8Array(8 + 16); // header (no version) + 2 entities
      const view = new DataView(buffer.buffer);

      // Header (no version byte)
      view.setUint32(0, 2, true); // count
      view.setUint32(4, 500, true); // timestamp

      // Entity 1: id=10, value=1.5
      view.setUint32(8, 10, true);
      view.setFloat32(12, 1.5, true);

      // Entity 2: id=20, value=2.5
      view.setUint32(16, 20, true);
      view.setFloat32(20, 2.5, true);

      const decoded = codec.decodeLegacy(buffer);

      expect(decoded.version).toBe(0); // Legacy format has version 0
      expect(decoded.timestamp).toBe(500);
      expect(decoded.items).toHaveLength(2);
      expect(decoded.items[0].id).toBe(10);
      expect(decoded.items[0].value).toBeCloseTo(1.5, 2);
      expect(decoded.items[1].id).toBe(20);
      expect(decoded.items[1].value).toBeCloseTo(2.5, 2);
    });

    it('should throw error if legacy buffer is too short', () => {
      const codec = new ProtocolCodec(new TestEntityDecoder(), 1);
      const buffer = new Uint8Array(5); // Only 5 bytes, need at least 8

      expect(() => codec.decodeLegacy(buffer)).toThrow('Buffer too short');
    });

    it('should decode empty legacy collection', () => {
      const codec = new ProtocolCodec(new TestEntityDecoder(), 1);

      // Create legacy buffer with 0 entities
      const buffer = new Uint8Array(8);
      const view = new DataView(buffer.buffer);
      view.setUint32(0, 0, true); // count 0
      view.setUint32(4, 999, true); // timestamp

      const decoded = codec.decodeLegacy(buffer);

      expect(decoded.version).toBe(0);
      expect(decoded.timestamp).toBe(999);
      expect(decoded.items).toHaveLength(0);
    });
  });

  describe('version handling', () => {
    it('should work with different version numbers', () => {
      const codecV1 = new ProtocolCodec(new TestEntityDecoder(), 1);
      const codecV5 = new ProtocolCodec(new TestEntityDecoder(), 5);
      const codecV255 = new ProtocolCodec(new TestEntityDecoder(), 255);

      // Create buffers with different versions
      const bufferV1 = new Uint8Array(9);
      const bufferV5 = new Uint8Array(9);
      const bufferV255 = new Uint8Array(9);

      bufferV1[0] = 1;
      bufferV5[0] = 5;
      bufferV255[0] = 255;

      // Set count and timestamp to 0 for all
      for (const buf of [bufferV1, bufferV5, bufferV255]) {
        const view = new DataView(buf.buffer);
        view.setUint32(1, 0, true);
        view.setUint32(5, 0, true);
      }

      // Each codec should decode its own version
      expect(() => codecV1.decode(bufferV1)).not.toThrow();
      expect(() => codecV5.decode(bufferV5)).not.toThrow();
      expect(() => codecV255.decode(bufferV255)).not.toThrow();

      // But reject other versions
      expect(() => codecV1.decode(bufferV5)).toThrow('Version mismatch');
      expect(() => codecV5.decode(bufferV1)).toThrow('Version mismatch');
      expect(() => codecV1.decode(bufferV255)).toThrow('Version mismatch');
    });
  });
});
