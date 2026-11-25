# Working in this repository

Three standing principles hold for every bmo repository, this one included:

- **(a) Each service runs on its own,** with no context bleeding in from other repositories: it builds, tests
  and runs from its own repository, data and settings, and never reads another repository's code, files,
  data folders or tables.
- **(b) Services talk to each other only through documented APIs** (HTTP routes, events, a documented
  command), never by importing another service's code.
- **(c) Upstream never intrudes on downstream:** a service calls only what the service below it documents,
  never reaches into its data or decides what is that service's to decide, and a base library never depends
  on a service.

This repository is the base library `bmo-text`: the reading and writing of markdown and text that the
chat services, the console and the archive share. It ships one Rust crate at the root, where every reading is
written, and one TypeScript package in `package/`, which is that crate built to WebAssembly under thin typed
modules. The crate's binary, run as `bmo-text mcp`, is the MCP server of the `bmo-text` harness plugin in
`plugin/`. It has no HTTP server, no deployment and no data of its own, and every bmo repository may
depend on it directly.

## Rules

- **`docs/` is canonical.** `docs/design.md` states the shape and the boundary decisions, `docs/api.md`
  every public item, `docs/boundary.md` what the library exposes and calls, `docs/inventory.md` what it was
  built from. Read the one covering a change first. A change that would make it wrong repairs the document
  first, stating what is true now, and the code is then written to match.
- **The library depends on nothing `bmo-*`.** Both manifests name registry dependencies only; the
  repository test fails on any other. What differs per consumer (words a reader sees, a part's English
  policy, a glossary's path) is passed in by the consumer, with a safe default here.
- **Tests live in `tests/`:** `tests/` for the crate, `package/tests/` for the package. No test module sits
  beside runtime code.
- **English only** in code, comments, strings, tests and documents, and in commit messages. A message the
  library returns is an English sentence; a machine-read outcome is a status code or a kebab-case code.
- **Korean only where the code inherently reads Korean** (a Hangul-aware parser, a Korean line of a
  glossary, a test whose point is how Hangul is read). Such Korean is written plainly, never as an escape
  sequence or a char code, and every file holding it is counted in `korean-ledger.json`.
- **No names of agents, owners or hosts, and no personal paths** in code, documents or commits. Write the
  role instead: a consumer, a reader, the owner. `docs/PERSONAL-DATA.md` states the whole rule and the
  check that holds the repository to it.
- **No directory named after its parent** (no `x/x`) and **no folder named for a language.** The crate's
  manifest and lockfile are the only ones at the root; `package/` is a project of its own with its manifest
  and lockfile.
- **Documents state what is true now**: no change history, no quoted source and no file-and-line pins.

## A change from a repository below

A repository below that needs a change this library does not serve sends it as a pull request against this
repository's GitHub repository, from a branch made from this repository's main. Its job never commits to
this main and keeps no copy of the change.

The change is written as the general case, with this library's own tests. Before it is opened and again in
review it is held to the rule that it names no repository that asked for it, no provider or unit of one, no
branch for one unit, and no data or test fixture only the asker uses.

Whoever holds this repository's work reviews the pull request and merges it or sends it back with the reason
on it; a review service may assist. A merge moves no version (`docs/conventions.md` in bmo-chat, 「Versions」: a version moves only at a release the owner declares); the repository that asked builds against the merged main through its path dependency and resumes the part it stopped.

## Checks

- Rust, from the root: `cargo check --all-targets`, `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo fmt --check` (108 columns, 2-space indentation, as `rustfmt.toml` sets), then `cargo test`.
- TypeScript, from `package/`: `bun run check`, the static check and the tests together. After any change to
  the crate, `bun run build` first: the package's tests fail on a WebAssembly build older than the crate.
- `testbed.toml` declares the quick checks and the setup for the testbed, which checks and tests the crate
  as the root project and the package by its `check` script.
- The version is the set's, read off the git tag when a build is made: the testbed hands it to the build as
  `BMO_VERSION`, and the MCP server names it (bmo-chat's `docs/conventions.md`, 「Versions」). The crate's,
  the package's and the plugin's manifest versions are placeholders that never move.
