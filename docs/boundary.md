# Boundary

## What this repository exposes

- **The Rust crate `bmo-text`,** at the repository's root, taken by a consumer as a path or git
  dependency. Its public modules and items are stated in `docs/api.md`.
- **The TypeScript package `bmo-text`,** in `package/`, taken by a consumer as a dependency and imported
  one module at a time as `bmo-text/<module>`. Its exports are stated in `docs/api.md`.
- **The MCP server** the crate's binary starts as `bmo-text mcp`, over its standard input and output, and
  its seven tools, stated in `docs/api.md`.
- **The harness plugin `bmo-text`,** in `plugin/`: its manifest, the MCP configuration starting the server,
  and the `bmo-text` skill.
- **The cases** both sides are held to, one file per module under `cases/`, named in `docs/api.md`.

The version is the set's, read off the git tag when a build is made; the manifests' versions are placeholders
that never move. A consumer pins no version: it depends on this repository by path and is built against the
checkout beside it.

## What this repository calls

Nothing. It runs no HTTP server, sends no event, reads no route, no data folder and no table, and depends
on no repository: both manifests name registry crates and packages only. The MCP server answers only the
harness that started it, and reads no file but the glossary in the folder a call names. Every value that
differs per consumer is a parameter the consumer passes in (`docs/design.md`, 「What the consumer decides」).

## Who may depend on it

Any bmo repository, service or base library alike, as one of the base libraries the testbed's
`dependencies` check allows, and any coding harness, through the plugin. It knows none of them.
