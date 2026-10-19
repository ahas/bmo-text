# Boundary

## What this repository exposes

- **The Rust crate `bmo-markup`,** in `rust/`, taken by a consumer as a path or git dependency. Its public
  modules and items are stated in `docs/api.md`.
- **The TypeScript package `bmo-markup`,** in `ts/`, taken by a consumer as a dependency and imported one
  module at a time as `bmo-markup/<module>`. Its exports are stated in `docs/api.md`.
- **The data files** the package's readings and tests stand on, named in `docs/api.md`.

The version both carry is the repository's, in `VERSION`. A consumer pins the version it was built
against and moves to a new major only by its own change.

## What this repository calls

Nothing. It runs no server, sends no event, reads no route, no data folder and no table, and depends on
no repository: both manifests name registry crates and packages only. Every value that differs per
consumer is a parameter the consumer passes in (`docs/design.md`, 「What the consumer decides」).

## Who may depend on it

Any bmo-next repository, service or base library alike, as one of the base libraries the testbed's
`dependencies` check allows. It knows none of them.
