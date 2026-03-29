# Account Kernel

## Purpose

The goal is to make authorization explicit, modular, and replay-safe before any `1.0` API freeze.

## Core Model

The account subsystem is now organized around:

- `AccountKernel`
  - the orchestrator that runs signer verification, policy checks, and replay protection
- signer interfaces
  - `AccountSigner`
  - base implementations: direct owner auth, session auth, secp256r1 passkey auth
- policy interfaces
  - generic `Policy<C>`
  - base implementations for intent expiry, session enforcement, active device checks, and guardian checks
- signed intent schema
  - `SignedIntent`, `SignerRef`, `IntentProof`
- structured auth results
  - `AuthResult`, `AuthMethod`

## Signed Intent Schema

`SignedIntent` binds:

- target account
- signer reference
- action payload
- nonce
- expiry
- deterministic `action_hash`
- proof material

The deterministic hash is derived from:

- nonce
- expiry
- signer identity fields
- action system name
- action bytes

## Replay Protection

Cougr uses two replay domains:

- per-account nonce tracking for direct owner auth and passkey auth
- per-session nonce tracking for session intents

The replay implementation lives in:

- [src/accounts/replay.rs](../src/accounts/replay.rs)
- [src/accounts/storage.rs](../src/accounts/storage.rs)

## Session Model

Session state now includes:

- unique `key_id`
- scoped allowed actions
- operation budget
- expiration timestamp
- `next_nonce`

Session enforcement requires all of:

- session exists
- action is in scope
- session is not expired
- operation budget remains
- intent nonce matches `next_nonce`

On success the session consumes one operation and advances `next_nonce`.

## Signers

Current base signer implementations:

- direct owner signer
  - uses `require_auth`
- session signer
  - explicit non-fallback session path evaluated by the kernel
- secp256r1 passkey signer
  - verifies signatures against registered passkeys

## Policies

The policy layer is intentionally reusable across account features.

Current base policies:

- `IntentExpiryPolicy`
- `SessionPolicy`
- `ActiveDevicePolicy`
- `GuardianPolicy`

This is how device and recovery support now live under the same policy model instead of ad hoc checks.

## Auth Results

`AuthResult` returns structured information instead of only `Result<(), AccountError>`.

Current fields:

- method used
- nonce consumed
- session key id, when applicable
- remaining operations, when applicable

## GameWorld Integration

`GameWorld` now exposes:

- direct owner auth via `execute_authorized`
- explicit kernel execution via `execute_intent`
- explicit session execution via `execute_with_active_session`

This removes the old dependency on implicit session-to-owner fallback from the primary auth path.
