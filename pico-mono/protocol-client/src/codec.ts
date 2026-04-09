/**
 * Generic Protocol Codec with Versioning
 * 
 * This module provides a universal codec for serializing and deserializing
 * collections of items with versioning support.
 * 
 * # Format
 * 
 * New format with version:
 * ```
 * [VERSION:u8][COUNT:u32][TIMESTAMP:u32][ITEMS...]
 * ```
 * 
 * Legacy format (no version):
 * ```
 * [COUNT:u32][TIMESTAMP:u32][ITEMS...]
 * ```
 * 
 * @example
 * ```typescript
 * import { ProtocolCodec, EntityStateDecoder } from '@picoTaas/protocol-client';
 * 
 * const codec = new ProtocolCodec(new EntityStateDecoder(), 1);
 * const decoded = codec.decode(buffer);
 * console.log(decoded.version, decoded.timestamp, decoded.items);
 * ```
 */

import type { BinaryDecoder, DecodedMessage } from './types';

/**
 * Generic Protocol Codec
 * 
 * Universal codec for serializing and deserializing collections of items
 * with versioning support.
 * 
 * @template T - The type of items to encode/decode. Must have a corresponding
 *               BinaryDecoder implementation for decoding.
 * 
 * @example
 * ```typescript
 * const codec = new ProtocolCodec(new EntityStateDecoder(), 1);
 * const decoded = codec.decode(buffer);
 * ```
 */
export class ProtocolCodec<T> {
  /**
   * Create a new codec with the specified version.
   * 
   * @param decoder - Binary decoder for individual items
   * @param expectedVersion - Protocol version (typically 1 for new format)
   * 
   * @example
   * ```typescript
   * const codec = new ProtocolCodec(new EntityStateDecoder(), 1);
   * ```
   */
  constructor(
    private decoder: BinaryDecoder<T>,
    private expectedVersion: number = 1
  ) {}

  /**
   * Decode a collection of items with versioning.
   * 
   * Format: `[VERSION:u8][COUNT:u32][TIMESTAMP:u32][ITEMS...]`
   * 
   * @param buffer - Binary buffer with encoded data
   * @returns Decoded message with version, timestamp, and items
   * @throws Error if buffer is too short or version mismatch
   * 
   * @example
   * ```typescript
   * const codec = new ProtocolCodec(new EntityStateDecoder(), 1);
   * const decoded = codec.decode(buffer);
   * 
   * console.log(decoded.version);    // 1
   * console.log(decoded.timestamp);  // Unix timestamp in ms
   * console.log(decoded.items);      // Array of decoded items
   * ```
   */
  decode(buffer: Uint8Array): DecodedMessage<T> {
    if (buffer.length < 9) {
      throw new Error(
        `Buffer too short: expected at least 9 bytes, got ${buffer.length}`
      );
    }

    const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);

    // Protocol version (1 byte)
    const version = view.getUint8(0);
    if (version !== this.expectedVersion) {
      throw new Error(
        `Version mismatch: expected ${this.expectedVersion}, got ${version}`
      );
    }

    // Item count (4 bytes)
    const count = view.getUint32(1, true);

    // Timestamp (4 bytes)
    const timestamp = view.getUint32(5, true);

    // Decode items
    const items: T[] = [];
    let offset = 9;

    for (let i = 0; i < count; i++) {
      if (offset + this.decoder.byteSize > buffer.length) {
        throw new Error(
          `Buffer too short for item ${i}: expected ${offset + this.decoder.byteSize} bytes, got ${buffer.length}`
        );
      }

      const itemBuffer = buffer.slice(offset, offset + this.decoder.byteSize);
      items.push(this.decoder.decode(itemBuffer));
      offset += this.decoder.byteSize;
    }

    return { version, timestamp, items };
  }

  /**
   * Decode in legacy format (no version) for backward compatibility.
   * 
   * Format: `[COUNT:u32][TIMESTAMP:u32][ITEMS...]`
   * 
   * @param buffer - Binary buffer with encoded data (legacy format)
   * @returns Decoded message with version=0, timestamp, and items
   * @throws Error if buffer is too short or data is invalid
   * 
   * @example
   * ```typescript
   * const codec = new ProtocolCodec(new EntityStateDecoder(), 1);
   * const decoded = codec.decodeLegacy(legacyBuffer);
   * 
   * console.log(decoded.version);    // 0 (legacy format)
   * console.log(decoded.timestamp);  // Unix timestamp in ms
   * console.log(decoded.items);      // Array of decoded items
   * ```
   */
  decodeLegacy(buffer: Uint8Array): DecodedMessage<T> {
    if (buffer.length < 8) {
      throw new Error(
        `Buffer too short: expected at least 8 bytes, got ${buffer.length}`
      );
    }

    const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);

    // Item count (4 bytes)
    const count = view.getUint32(0, true);

    // Timestamp (4 bytes)
    const timestamp = view.getUint32(4, true);

    // Decode items
    const items: T[] = [];
    let offset = 8;

    for (let i = 0; i < count; i++) {
      if (offset + this.decoder.byteSize > buffer.length) {
        throw new Error(
          `Buffer too short for item ${i}: expected ${offset + this.decoder.byteSize} bytes, got ${buffer.length}`
        );
      }

      const itemBuffer = buffer.slice(offset, offset + this.decoder.byteSize);
      items.push(this.decoder.decode(itemBuffer));
      offset += this.decoder.byteSize;
    }

    return { version: 0, timestamp, items };
  }
}
