/**
 * @picoTaas/protocol-client
 * 
 * Binary protocol decoder for picoTaas simulation engine.
 * 
 * This package provides a generic codec for decoding binary protocol messages
 * with versioning support.
 * 
 * @example
 * ```typescript
 * import { ProtocolCodec, EntityStateDecoder } from '@picoTaas/protocol-client';
 * 
 * const codec = new ProtocolCodec(new EntityStateDecoder(), 1);
 * const decoded = codec.decode(buffer);
 * ```
 */

export { ProtocolCodec } from './codec';
export type { BinaryDecoder, DecodedMessage } from './types';
export { EntityStateDecoder } from './decoders/entity-state';
export type { EntityState } from './decoders/entity-state';
