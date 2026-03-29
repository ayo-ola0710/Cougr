# Cougr Maturity Model

## Purpose

This document defines how Cougr classifies public surfaces before `1.0`.

The goal is straightforward: stability, documentation, and compatibility promises must match the actual implementation.

## Levels

### Stable

Stable features are:

- SemVer-protected
- documented with invariants and intended usage
- covered by focused tests, including negative paths where relevant
- safe to present as part of Cougr's long-term public contract

Stable features must not contain:

- placeholder identifiers
- incomplete public behavior
- undocumented fallback logic in security-critical paths
- undocumented storage or auth assumptions

### Beta

Beta features are:

- usable and actively supported
- expected to evolve before `1.0`
- covered by tests, but not yet frozen in API or guarantees

Beta is the right classification for modules that have real implementation value but still need one or more of:

- tighter interface design
- stronger invariants
- stronger security posture
- compatibility cleanup

### Experimental

Experimental features are:

- exploratory or fast-moving
- not part of Cougr's stable promise
- allowed to change or be removed without compatibility guarantees

Experimental is the default for features where:

- the implementation contract is still incomplete
- external security assumptions are still being clarified
- ecosystem support is still emerging
- the API shape is not yet trustworthy enough to freeze

## Current Baseline

| Surface | Status | Notes |
|---|---|---|
| ECS runtime, worlds, storage, scheduling | Beta | Broadly usable, but the public surface still needs narrowing before `1.0` |
| Accounts and smart-account flows | Beta | Valuable direction, but auth kernel and session enforcement still need redesign |
| Commitments, commit-reveal, and Merkle utilities | Beta | Closer to defensible than advanced proof-verification flows |
| Advanced ZK verification and confidential abstractions | Experimental | Do not treat as stable production primitives yet |
| Testing helpers | Non-stable support surface | Intended only for tests or explicit test utility consumers |

## Rules for Public Surfaces

- Public APIs that are placeholders must be downgraded or removed from the stable story.
- Testing-only modules should not remain in the default stable surface.
- Documentation must name maturity honestly.
- New security-sensitive features should default to Beta or Experimental unless proven otherwise.

## Promotion Criteria

A feature should move toward Stable only when:

1. its API is intentionally designed and unlikely to churn
2. its invariants are written down
3. its failure modes are documented
4. it has negative-path tests where relevant
5. its compatibility impact is understood

## Demotion Criteria

A public surface should be demoted from the stable story when:

- it contains placeholder logic
- docs overstate what it guarantees
- replay, authorization, or verification semantics are incomplete
- maintainers are not prepared to keep compatibility promises for it
