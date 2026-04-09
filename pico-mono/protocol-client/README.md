# @picoTaas/protocol-client

Binary protocol decoder for picoTaas simulation engine.

## Overview

This package provides a TypeScript implementation of the binary protocol decoder for the picoTaas simulation engine. It supports versioned protocol messages and provides a generic, extensible codec architecture.

## Features

- **Generic Codec**: `ProtocolCodec<T>` works with any type implementing `BinaryDecoder<T>`
- **Protocol Versioning**: Built-in support for protocol version checking
- **Type-Safe**: Full TypeScript type definitions
- **Extensible**: Easy to add custom decoders for new message types
- **Backward Compatible**: Supports legacy format decoding

## Installation

```bash
npm install @picoTaas/protocol-client
```

## Quick Start

```typescript
import { ProtocolCodec, EntityStateDecoder } from '@picoTaas/protocol-client';

// Create a codec with EntityState decoder
const codec = new ProtocolCodec(new EntityStateDecoder(), 1);

// Decode a binary message
const buffer = new Uint8Array([/* binary data */]);
const message = codec.decode(buffer);

console.log(message.version);    // Protocol version
console.log(message.timestamp);  // Message timestamp
console.log(message.items);      // Decoded entities
```

## API Reference

### `ProtocolCodec<T>`

Generic codec for encoding/decoding binary protocol messages.

**Constructor:**
```typescript
constructor(decoder: BinaryDecoder<T>, expectedVersion: number = 1)
```

**Methods:**
- `decode(buffer: Uint8Array): DecodedMessage<T>` - Decode versioned message
- `decodeLegacy(buffer: Uint8Array): DecodedMessage<T>` - Decode legacy format (no version byte)

### `BinaryDecoder<T>`

Interface for custom decoders.

```typescript
interface BinaryDecoder<T> {
  decode(buffer: Uint8Array): T;
  readonly byteSize: number;
}
```

### `EntityStateDecoder`

Built-in decoder for entity state messages.

```typescript
interface EntityState {
  id: number;
  x: number;
  y: number;
  rotation: number;
  costume_id: number;
}
```

## Protocol Format

### Versioned Format (v1+)

```
[VERSION:1][COUNT:4][TIMESTAMP:4][ITEMS...]
```

- `VERSION`: Protocol version (1 byte, uint8)
- `COUNT`: Number of items (4 bytes, uint32 LE)
- `TIMESTAMP`: Unix timestamp in milliseconds (4 bytes, uint32 LE)
- `ITEMS`: Serialized items (variable length)

### Legacy Format (v0)

```
[COUNT:4][TIMESTAMP:4][ITEMS...]
```

No version byte. Use `decodeLegacy()` to decode.

## Creating Custom Decoders

```typescript
import { BinaryDecoder } from '@picoTaas/protocol-client';

interface MyData {
  value: number;
}

class MyDataDecoder implements BinaryDecoder<MyData> {
  readonly byteSize = 4;

  decode(buffer: Uint8Array): MyData {
    const view = new DataView(buffer.buffer, buffer.byteOffset);
    return {
      value: view.getUint32(0, true) // little-endian
    };
  }
}

// Use with codec
const codec = new ProtocolCodec(new MyDataDecoder(), 1);
```

## License

MIT
