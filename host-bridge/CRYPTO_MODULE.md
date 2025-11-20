# Host Bridge CRYPTO Module

## Overview

The CRYPTO module provides cryptographic operations for the Frame Framework, including:
- Cryptographically secure random number generation
- Password hashing (bcrypt and argon2)
- Password verification with constant-time comparison
- JWT signing and verification
- JWT decoding without verification (for debugging)

## Functions

### `host:crypto.random`

Generate cryptographically secure random bytes using OS entropy.

**Request:**
```json
{
  "fn": "host:crypto.random",
  "args": {
    "bytes": 32
  }
}
```

**Response:**
```json
{
  "ok": true,
  "data": "base64encodedrandombytes=="
}
```

**Parameters:**
- `bytes` (integer): Number of random bytes to generate (1 - 1,048,576)

**Error Codes:**
- `VALIDATION_ERROR`: Invalid byte count
- `CRYPTO_ERROR`: Random generation failed

---

### `host:crypto.hash`

Hash a password using bcrypt or argon2.

**Request:**
```json
{
  "fn": "host:crypto.hash",
  "args": {
    "password": "secret123",
    "algorithm": "bcrypt",
    "cost": 12
  }
}
```

**Response:**
```json
{
  "ok": true,
  "data": "$2b$12$hashvalue..."
}
```

**Parameters:**
- `password` (string): Password to hash (1 - 1,048,576 bytes, max 72 for bcrypt)
- `algorithm` (string): "bcrypt", "argon2", or "argon2id"
- `cost` (integer, optional): Bcrypt cost factor (10-14, default 12)

**Algorithms:**
- **bcrypt**: Traditional password hashing, cost factor 10-14 (default 12)
  - Maximum password length: 72 bytes
  - Output format: `$2b$<cost>$<salt+hash>`

- **argon2**: Modern password hashing (argon2id variant)
  - No practical password length limit (1MB max)
  - Output format: `$argon2id$<params>$<salt>$<hash>`

**Error Codes:**
- `VALIDATION_ERROR`: Empty password, invalid cost, or password too long
- `ALGORITHM_ERROR`: Unsupported algorithm
- `CRYPTO_ERROR`: Hashing operation failed

**Security Notes:**
- Uses cryptographically secure random salts
- Bcrypt cost 12 provides ~250ms computation time (recommended for production)
- Argon2id provides best resistance against GPU/ASIC attacks

---

### `host:crypto.verify`

Verify a password against a hash using constant-time comparison.

**Request:**
```json
{
  "fn": "host:crypto.verify",
  "args": {
    "password": "secret123",
    "hash": "$2b$12$..."
  }
}
```

**Response (match):**
```json
{
  "ok": true,
  "data": true
}
```

**Response (no match):**
```json
{
  "ok": true,
  "data": false
}
```

**Parameters:**
- `password` (string): Password to verify
- `hash` (string): Hash to verify against (bcrypt or argon2 format)

**Algorithm Detection:**
- Automatically detects algorithm from hash format
- `$2...` → bcrypt
- `$argon2...` → argon2

**Error Codes:**
- `VALIDATION_ERROR`: Empty password/hash or password too long for bcrypt
- `ALGORITHM_ERROR`: Unsupported hash format
- `CRYPTO_ERROR`: Verification operation failed

**Security Notes:**
- Uses constant-time comparison to prevent timing attacks
- Both correct and incorrect passwords take the same time to verify

---

### `host:crypto.sign`

Sign a JWT token with HMAC or RSA algorithms.

**Request:**
```json
{
  "fn": "host:crypto.sign",
  "args": {
    "payload": {
      "userId": 123,
      "role": "admin",
      "exp": 1735689600
    },
    "secret": "your-secret-key",
    "algorithm": "HS256"
  }
}
```

**Response:**
```json
{
  "ok": true,
  "data": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Parameters:**
- `payload` (object): JWT claims (must be an object)
- `secret` (string): Secret key for HMAC algorithms
- `algorithm` (string): "HS256", "HS384", "HS512", or "RS256"

**Standard Claims:**
- `exp` (integer): Expiration time (Unix timestamp)
- `iat` (integer): Issued at time (Unix timestamp)
- `nbf` (integer): Not before time (Unix timestamp)
- `sub` (string/integer): Subject (user ID)
- `aud` (string/array): Audience
- `iss` (string): Issuer

**Supported Algorithms:**
- **HS256**: HMAC with SHA-256 (recommended for most use cases)
- **HS384**: HMAC with SHA-384
- **HS512**: HMAC with SHA-512
- **RS256**: RSA with SHA-256 (requires RSA key pair)

**Error Codes:**
- `VALIDATION_ERROR`: Empty secret or non-object payload
- `ALGORITHM_ERROR`: Unsupported algorithm
- `CRYPTO_ERROR`: Signing operation failed

---

### `host:crypto.verify_jwt`

Verify and decode a JWT token.

**Request:**
```json
{
  "fn": "host:crypto.verify_jwt",
  "args": {
    "token": "eyJhbGc...",
    "secret": "your-secret-key"
  }
}
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "userId": 123,
    "role": "admin",
    "exp": 1735689600,
    "iat": 1735603200
  }
}
```

**Parameters:**
- `token` (string): JWT token to verify
- `secret` (string): Secret key used to sign the token

**Verification:**
- Validates signature using the secret
- Checks expiration time (`exp` claim)
- Allows 60 seconds clock skew
- Tries multiple HMAC algorithms to prevent algorithm confusion attacks

**Error Codes:**
- `VALIDATION_ERROR`: Empty token or secret
- `AUTH_ERROR`: Invalid signature, expired token, or verification failed

**Security Notes:**
- Always verify tokens before trusting claims
- Tokens are rejected if expired (respects `exp` claim)
- 60-second leeway for clock skew between servers

---

### `host:crypto.decode_jwt`

Decode a JWT token without verification (for debugging only).

**Request:**
```json
{
  "fn": "host:crypto.decode_jwt",
  "args": {
    "token": "eyJhbGc..."
  }
}
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "header": {
      "alg": "HS256",
      "typ": "JWT"
    },
    "payload": {
      "userId": 123,
      "role": "admin",
      "exp": 1735689600
    }
  }
}
```

**Parameters:**
- `token` (string): JWT token to decode

**Warning:**
⚠️ This function does NOT verify the signature. Use only for debugging.
Never use decoded data for authorization without verification.

**Error Codes:**
- `VALIDATION_ERROR`: Empty token
- `CRYPTO_ERROR`: Invalid JWT format

---

## Error Response Format

All errors follow the standard bridge error format:

```json
{
  "ok": false,
  "err": {
    "code": "ERROR_CODE",
    "message": "Human-readable error description",
    "details": {}
  }
}
```

### Error Codes

| Code | Description |
|------|-------------|
| `CRYPTO_ERROR` | General cryptographic operation failure |
| `VALIDATION_ERROR` | Invalid input parameters |
| `ALGORITHM_ERROR` | Unsupported or invalid algorithm |
| `AUTH_ERROR` | JWT verification failure |

---

## Security Best Practices

### Password Hashing

1. **Use bcrypt cost 12** for production (balances security and performance)
2. **Never store plain text passwords**
3. **Use argon2** for higher security requirements
4. **Limit password length** to prevent DoS attacks (72 bytes for bcrypt)

### JWT Tokens

1. **Use short expiration times** (15-60 minutes for access tokens)
2. **Use refresh tokens** for long-lived sessions
3. **Always verify tokens** before trusting claims
4. **Use HS256** for most use cases (simpler, faster)
5. **Rotate secrets regularly**
6. **Store secrets in environment variables**, never in code

### Random Generation

1. **Use at least 32 bytes** for secrets and session IDs
2. **Never use Math.random()** or other non-cryptographic RNGs
3. **Encode as base64** for safe transmission

---

## Examples

### Complete Authentication Flow

```javascript
// 1. User registration - hash password
const hashResult = await bridge.call("crypto", "hash", {
  password: userPassword,
  algorithm: "bcrypt",
  cost: 12
});
const hash = hashResult.data;

// Store hash in database
await db.query("INSERT INTO users (email, password_hash) VALUES ($1, $2)",
  [email, hash]);

// 2. User login - verify password
const user = await db.query("SELECT * FROM users WHERE email = $1", [email]);

const verifyResult = await bridge.call("crypto", "verify", {
  password: userPassword,
  hash: user.password_hash
});

if (verifyResult.data) {
  // 3. Generate JWT token
  const tokenResult = await bridge.call("crypto", "sign", {
    payload: {
      userId: user.id,
      email: user.email,
      role: user.role,
      exp: Math.floor(Date.now() / 1000) + (60 * 60)  // 1 hour
    },
    secret: process.env.JWT_SECRET,
    algorithm: "HS256"
  });

  return { token: tokenResult.data };
}

// 4. Verify token on subsequent requests
const verifyTokenResult = await bridge.call("crypto", "verify_jwt", {
  token: authHeader.replace("Bearer ", ""),
  secret: process.env.JWT_SECRET
});

if (verifyTokenResult.ok) {
  const claims = verifyTokenResult.data;
  // User is authenticated with userId: claims.userId
}
```

### Session ID Generation

```javascript
const randomResult = await bridge.call("crypto", "random", {
  bytes: 32
});

const sessionId = randomResult.data;  // base64-encoded 32-byte random value
```

### Password Migration (bcrypt to argon2)

```javascript
// Verify with old bcrypt hash
const verifyResult = await bridge.call("crypto", "verify", {
  password: userPassword,
  hash: user.bcrypt_hash
});

if (verifyResult.data) {
  // Re-hash with argon2
  const newHashResult = await bridge.call("crypto", "hash", {
    password: userPassword,
    algorithm: "argon2"
  });

  // Update database
  await db.query("UPDATE users SET password_hash = $1 WHERE id = $2",
    [newHashResult.data, user.id]);
}
```

---

## Testing

Run the crypto module tests:

```bash
cargo test crypto --lib
```

Run integration tests:

```bash
cargo test --test crypto_integration_test
```

---

## Implementation Details

### Dependencies

- `bcrypt` v0.15: bcrypt password hashing
- `argon2` v0.5: argon2 password hashing
- `jsonwebtoken` v9: JWT signing and verification
- `rand` v0.8: Cryptographically secure random generation
- `base64` v0.22: Base64 encoding/decoding

### Performance

| Operation | Time (approx.) |
|-----------|----------------|
| Random (32 bytes) | < 1ms |
| bcrypt (cost 10) | ~65ms |
| bcrypt (cost 12) | ~250ms |
| argon2 (default) | ~100ms |
| JWT sign | < 1ms |
| JWT verify | < 1ms |

**Note**: Hashing times are intentional (slows down brute-force attacks)

---

## Version

Module version: 1.0.0
Frame version: 1.0.0
Last updated: 2025-01-19
