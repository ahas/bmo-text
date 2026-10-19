# bmo-markup

The base library for reading and writing markdown and text the way a chat room, the console and the
archive read it: markdown into Telegram's HTML and plain text, the structure check of a piece of markup,
the fold and the division into messages, web addresses in running text, the dash rewrite, the check of a
text against a glossary of banned terms, the answer ceiling, the structure of a markdown document, a
search finding's structure lines and an HTML page read into lines. Lengths are counted in UTF-16 units and
spaces and trims follow JavaScript, so the Rust crate and the TypeScript package give one answer for one
text.

| Folder | What it holds |
|---|---|
| `docs/` | The design, the public API, the boundary and the inventory of what the library was built from |
| `rust/` | The Rust crate `bmo-markup`, its sources and its tests |
| `ts/` | The TypeScript package `bmo-markup`, its modules, its tests and their data files |

The library depends on nothing but registry packages and has no server or deployment. A consumer takes
the crate by path or git and the package as a dependency, and passes in whatever is its own to decide:
the words a reader sees, its English policy, the path of its glossary.

Checks and tests are declared in `testbed.toml`; `CLAUDE.md` names the commands.
