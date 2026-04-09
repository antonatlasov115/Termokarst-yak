/**
 * Binary decoder interface for protocol messages.
 * 
 * Implementations of this interface can decode binary data into typed objects.
 * Used by ProtocolCodec to decode individual items from protocol messages.
 * 
 * @template T - The type of object this decoder produces
 * 
 * @example
 * ```typescript
 * class MyDecoder implements BinaryDecoder<MyType> {
 *   readonly byteSize = 8;
 *   
 *   decode(buffer: Uint8Array): MyType {
 *     const view = new DataView(buffer.buffer, buffer.byteOffset);
 *     return {
 *       value: view.getUint32(0, true)
 *     };
 *   }
 * }
 * ```
 */
export interface BinaryDecoder<T> {
  /**
   * Decode a single item from a binary buffer.
   * 
   * @param buffer - Binary data to decode (must be at least byteSize bytes)
   * @returns Decoded object of type T
   * @throws Error if buffer is too short or data is invalid
   */
  decode(buffer: Uint8Array): T;

  /**
   * Size in bytes of a single encoded item.
   * 
   * This value is used to slice the buffer when decoding multiple items.
   */
  readonly byteSize: number;
}

/**
 * Decoded protocol message with metadata.
 * 
 * Represents a complete protocol message after decoding, including
 * version information, timestamp, and the decoded items.
 * 
 * @template T - The type of items in the message
 * 
 * @example
 * ```typescript
 * const message: DecodedMessage<EntityState> = {
 *   version: 1,
 *   timestamp: 1234567890,
 *   items: [
 *     { id: 1, x: 0, y: 0, rotation: 0, costume_id: 0 }
 *   ]
 * };
 * ```
 */
export interface DecodedMessage<T> {
  /**
   * Protocol version number.
   * 
   * Used to ensure compatibility between encoder and decoder.
   * Version 0 indicates legacy format (no version byte in buffer).
   */
  version: number;

  /**
   * Message timestamp in milliseconds since Unix epoch.
   * 
   * Represents when the message was encoded.
   */
  timestamp: number;

  /**
   * Decoded items from the message.
   * 
   * Array of objects decoded from the binary buffer.
   */
  items: T[];
}
