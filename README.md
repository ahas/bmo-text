# bmo-text

The base library for reading and writing markdown and text the way a chat room, the console and the
archive read it: markdown into Telegram's HTML and plain text, the structure check of a piece of markup,
the fold and the division into messages, web addresses in running text, the dash rewrite, the check of a
text against a glossary of banned terms, the answer ceiling, the structure of a markdown document, a
search finding's structure lines, the parts of a text handed to a translator and an HTML page read into lines.
The readings are written once, in the Rust crate; the TypeScript package is the crate built to WebAssembly,
loaded synchronously on import in Bun, Node and the browser alike. Lengths are counted in UTF-16 units unless
named `chars`, and spaces and trims follow JavaScript, so both give one answer for one input.

| Folder | What it holds |
|---|---|
| `docs/` | The design, the public API, the boundary and the inventory of what the library was built from |
| `src/` | The sources of the Rust crate `bmo-text`, whose manifest stands at the root |
| `tests/` | The crate's tests, and the repository test that holds the repository's own shape |
| `package/` | The TypeScript package `bmo-text`: its modules over the crate's WebAssembly build, the build in `wasm/`, and its tests |
| `cases/` | The cases both the crate and the package are held to, one file per module |

The library depends on nothing but registry packages and has no server or deployment. A consumer takes
the crate by path or git and the package as a dependency, and passes in whatever is its own to decide:
the words a reader sees, its English policy, the path of its glossary.

Checks and tests are declared in `testbed.toml`; `CLAUDE.md` names the commands.
