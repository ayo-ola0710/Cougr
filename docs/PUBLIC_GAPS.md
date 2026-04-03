# Public Surface Gaps

## Purpose

This document lists public-facing behaviors and constraints that remain intentionally outside Cougr's stable promise after the phase 0 foundation reset.

The goal is to keep the stable story honest. If a behavior is still evolving, security-sensitive, or not yet defensible as a long-term contract, it belongs here instead of being implied as stable.

## Current Gaps

### Accounts and Smart-Account Flows

Status: Beta

Remaining gaps:

- session semantics are implemented, but the long-term auth kernel is still scheduled for redesign in phase 1
- replay protection is not yet presented as a frozen cross-module contract
- fallback authorization behavior is documented, but it should still be reviewed carefully per integration

### Privacy and ZK

Status: Stable subset plus Experimental extensions

Remaining gaps:

- stable privacy primitives are intentionally narrow and do not imply stable advanced proof verification
- advanced proof-verification and confidential abstractions are explicitly Experimental
- proof-submission orchestration remains Beta where it depends on experimental verification flows

### Broad Top-Level Crate Surface

Status: Beta

Remaining gaps:

- the crate still exports more modules than the eventual stable golden path is likely to keep
- internals-heavy modules remain public in places where durable invariants are not yet fully documented

## Removed or Downgraded During Phase 0

- the deprecated top-level placeholder helper `query_entities` was removed from `src/lib.rs`
- `zk::testing` remains outside the default product contract and is only available for tests or `testutils`
- accounts and advanced privacy features are described as Beta or Experimental rather than stable production guarantees

## How To Use This List

Treat this document as the current boundary between:

- what Cougr can present as intentionally supported today
- what is still usable but evolving
- what must not be interpreted as part of a stable production contract
