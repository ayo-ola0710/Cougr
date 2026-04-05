# Release Status

## Purpose

This document is the high-signal answer to one question:

- if Cougr shipped `1.0.0` from the current tree, what would be true?

It is intentionally shorter than the full release checklist.

## Current Assessment

Cougr is close to a defendable `1.0.0` for its curated product surface, but
publication quality still depends on maintaining strict scope control between
the stable contract and the broader public repository.

That claim is intentionally narrow:

- `app` is the default runtime surface
- `SimpleWorld` and `ArchetypeWorld` are the defended Soroban-first backends
- `RuntimeWorld` and `RuntimeWorldMut` define the shared stable overlap
- `ops` is the preferred stable standards namespace
- `privacy::stable` is the preferred stable privacy namespace

## What Is Stable

- ECS onboarding/runtime contract
- `app`
- `ops` / `standards`
- `privacy::stable` / `zk::stable`
- the documented backend story around `SimpleWorld`, `ArchetypeWorld`, `RuntimeWorld`, and `RuntimeWorldMut`

## What Is Deliberately Not Frozen

- `auth` / `accounts`
- `game_world`

These remain public and supported, but they are not the product-learning path and they are not the strongest compatibility promise in the repo.

## What Is Explicitly Experimental

- `privacy::experimental`
- `zk::experimental`
- hazmat cryptographic helpers

## Remaining Honest Gaps

- the crate still exposes more public surface than the stable contract actually guarantees
- migration to the curated path is documented, but not enforced
- auth remains Beta
- advanced proof tooling remains Experimental
- example quality and manifest consistency still require tighter release discipline

## Release Readout

From a release perspective, the repo has moved from "promising but fuzzy" to
"narrower and much more explicit," but it still needs disciplined final cleanup
before publication.

The remaining risk is mostly scope control:

- keeping users on the curated path
- resisting the temptation to overstate Beta or Experimental surfaces
- preserving the freeze discipline already documented elsewhere
