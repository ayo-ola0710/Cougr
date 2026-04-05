# Release Status

## Purpose

This document is the high-signal answer to one question:

- if Cougr shipped `1.0.0` from the current tree, what would be true?

It is intentionally shorter than the full release checklist.

## Current Assessment

Cougr is now in a defendable `1.0.0` shape for its curated product surface.

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

- `legacy` for the older `World` / `System` path
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

## Release Readout

From a release perspective, the repo has moved from "promising but fuzzy" to "narrow, explicit, and defendable."

The remaining risk is mostly scope control:

- keeping users on the curated path
- resisting the temptation to overstate Beta or Experimental surfaces
- preserving the freeze discipline already documented elsewhere
