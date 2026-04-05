# Cougr Unsafe Invariants

## Purpose

This document gathers the architectural invariants that justify Cougr's unsafe-heavy areas for the `1.0` stable contract.

It does not replace inline `# Safety` docs in code. It records the higher-level assumptions maintainers are expected to preserve when changing these subsystems.

## Scope

The most important unsafe-heavy areas in the current repository are:

- world access splitting and interior mutability
- storage internals for table and blob-backed data
- command queue execution and deferred mutation
- entity and query internals inherited from lower-level ECS machinery

Representative files include:

- `src/world/unsafe_world_cell.rs`
- `src/world/command_queue.rs`
- `src/storage/table/column.rs`
- `src/storage/blob_vec.rs`
- `src/storage/blob_array.rs`

## Invariant 1: Exclusive Mutable World Access Must Remain Exclusive

When code obtains mutable access to the full world, no overlapping read or write access may be used through any other safe or unsafe handle for the same data.

This is the core contract behind:

- `World::as_unsafe_world_cell`
- `World::as_unsafe_world_cell_readonly`
- `UnsafeWorldCell::world_mut`

Breaking this invariant risks aliasing violations and unsound mutable access.

## Invariant 2: Readonly Unsafe Views Must Not Be Used To Mutate

Readonly unsafe world cells exist to support metadata or shared reads in contexts where compile-time borrowing is insufficient.

They must not be used to mutate world data or to materialize aliases that conflict with active mutable borrows.

## Invariant 3: Deferred Commands Must Not Observe Invalid Intermediate Layouts

Command queue execution relies on metadata and storage operations remaining internally consistent while deferred mutations are applied.

Changes to command packing, queue draining, or entity application order must preserve:

- command decoding correctness
- component layout correctness
- no use-after-move or use-after-free behavior in deferred storage paths

## Invariant 4: Storage Columns Must Preserve Type/Layout Agreements

Table and blob-backed storage internals assume that:

- the layout used for allocation matches the layout used for reads and writes
- element counts and capacities remain internally consistent
- drop behavior matches the stored type contract

Any change to allocation, resizing, or pointer math must be reviewed as a safety-sensitive change.

## Invariant 5: Entity and Query Internals Must Preserve Uniqueness and Liveness Assumptions

Entity sets, unique slices, fetch helpers, and archetype/query internals rely on:

- entity identity uniqueness
- correct component-location bookkeeping
- no stale pointers or stale indices surviving structural mutation

Refactors in these areas must preserve the assumptions documented inline in their `# Safety` sections.

## Invariant 6: Public Safety Docs And Architecture Docs Must Stay In Sync

If a change alters the meaning of an unsafe boundary, maintainers should update both:

- the local code-level `# Safety` documentation
- this architectural summary when the change affects repo-wide reasoning

## Review Checklist For Unsafe Changes

Before merging a change that touches unsafe-heavy code, confirm:

- aliasing assumptions still hold
- ownership and drop behavior are unchanged or intentionally reworked
- internal indices and capacities remain coherent across error paths
- tests cover the changed path
- docs still describe the real invariants
