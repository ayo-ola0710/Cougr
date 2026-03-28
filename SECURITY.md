# Security Policy

## Status

Cougr is currently pre-`1.0` software. The repository contains stable ideas, but not every public subsystem should be treated as production-ready.

Security-sensitive areas include:

- account authorization
- session lifecycle and replay protection
- persistent storage integrity
- proof verification and privacy primitives
- ECS mutation ordering where authorization depends on state transitions

## Maturity and Guarantees

Current guidance:

| Area | Status | Guidance |
|---|---|---|
| ECS runtime and storage | Beta | Suitable for active evaluation and internal use with validation |
| Accounts and smart-account flows | Beta | Do not assume full production guarantees without project-specific review |
| Privacy primitives | Beta | Commit-reveal and Merkle utilities are more mature than advanced proof flows |
| Advanced ZK verification | Experimental | Treat as non-stable until verification contracts and assumptions are fully hardened |

The latest maturity definitions live in [docs/MATURITY_MODEL.md](docs/MATURITY_MODEL.md).

## Threat Model Expectations

Cougr does not currently claim:

- external audit coverage
- formal verification
- full production guarantees across all auth and privacy paths
- stable compatibility guarantees for experimental modules

Before adopting Cougr in security-critical deployments, review at minimum:

- auth and signer flows
- replay handling
- session scope and revocation rules
- storage schema assumptions
- proof verification assumptions

## Reporting a Vulnerability

If you find a security issue:

1. Do not open a public issue with exploit details.
2. Report the issue privately to the project maintainers.
3. Include:
   - affected module
   - reproduction steps
   - impact assessment
   - version or commit information
   - suggested mitigation if available

Until a dedicated security contact is published, use the maintainer channels associated with this repository and clearly label the report as a security disclosure.

## Supported Versions

Because Cougr is pre-`1.0`, only the latest mainline development state should be assumed relevant for fixes unless a release branch explicitly says otherwise.

## Secure Contribution Expectations

Changes affecting auth, privacy, storage, or unsafe internals should include:

- updated invariants or trust assumptions
- negative-path tests
- compatibility notes when public behavior changes
- documentation changes when guarantees or maturity shift
