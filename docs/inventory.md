# Inventory: what this library takes from the old repository

This repository is built from the `markup/` folder of the old single bmo repository: the Rust crate
`bmo-markup` and the TypeScript package `@bmo/markup`, which read and wrote text for every other unit. It
starts with no history. This document lists every file, data file, policy value and test it takes, with
the count and the source each was read from, and what becomes of each.

## Sources

- The file list: `git ls-files markup` in the old repository, 42 files, which `find markup -type f` matches.
- The ties to other units: every `bmo_policy` use in the crate and every `@bmo/*` import and test preload in
  the package, each followed to its definition in the old repository.
- The outside tests: every test file outside `markup/` importing `bmo_markup` or `@bmo/markup`.
- The consumers: every import of either outside `markup/`, which sets the public API (`docs/api.md`).
- Archive document #301, 「원칙」, 「정책」, 「언어팩」 and 「옮기는 방식」, for what a base library may hold.

## The 42 files

| Kind | Count | Kept | Dropped |
|---|---|---|---|
| Rust modules | 9 | 9 | 0 |
| TypeScript runtime modules | 16 | 12 | 4 |
| TypeScript tests | 11 | 8 | 3 |
| Data files | 2 | 1 | 1 |
| Manifests | 4 | 2 | 2 |
| Total | 42 | 32 | 10 |

### Kept: the Rust crate, into `rust/`

Each module keeps its name under `rust/src/`. The eight that carried an inline test module have it moved
into `rust/tests/`, one file per module.

| Module | Inline tests | What changes in it |
|---|---|---|
| `lib` | no | The crate's doc comment names no agent |
| `dashes` | yes | Nothing beyond the move of its tests |
| `english` | yes | Built from the same two values as plain parameters (the set-aside pattern and the Latin letters per Hangul syllable) instead of a policy struct, with a strict default; its Hangul range written as literal characters |
| `format` | yes | The fault reasons are fixed English sentences; no policy crate and no fill helper |
| `js` | yes | Its hash test's Korean sample written plainly instead of as escapes |
| `markdown` | yes | Nothing beyond the move of its tests |
| `telegram` | yes | Nothing beyond the move of its tests |
| `web_address` | yes | Its Hangul ranges and its test's Korean sample written plainly instead of as escapes |
| `wording` | yes | Its test's Korean word written plainly instead of as escapes |

### Kept: the TypeScript package, into `ts/`

| Module | What changes in it |
|---|---|
| `answer-cap` | Formats its counts with its own thousands grouping instead of a shared util |
| `answer-language` | Nothing |
| `dashes` | Nothing beyond Korean in comments |
| `finding-markdown` | The words a reader sees for each part are passed in by the consumer; the English key is the default |
| `format` | The fault reasons are the same fixed English sentences as the crate's; no policy import, no fill helper |
| `html-reading` | Nothing |
| `markdown-structure` | Nothing |
| `markdown` | Nothing beyond Korean in comments |
| `telegram-parts` | Nothing |
| `translation-parts` | Nothing |
| `web-address` | Its Hangul ranges written as literal characters instead of escapes |
| `wording` | Loses its two checkout scans; reading a glossary takes the path from the caller |

The eight kept tests move into `ts/tests/`: `answer-cap`, `dashes`, `finding-markdown`, `format`,
`html-reading`, `markdown-structure`, `markdown` and `web-address`. Their example text follows the Korean
rule of `docs/design.md`, and names of people, agents and real places become neutral ones.

### Kept: data and manifests

- `answer-cap-cases.json`, the answer ceiling's cases, into `ts/tests/`. Its entries of a repeated Hangul
  syllable stay, because they are what proves a ceiling counts characters as a reader does rather than
  bytes; its other example words are English.
- `package.json` and `tsconfig.json`, rewritten for a package of its own (`docs/design.md`, 「Layout」).

### Dropped

| File | Why |
|---|---|
| `korean-in-code.ts`, its test | A scan of a whole checkout for Korean in code; the testbed's `korean` quick check owns it |
| `written-dashes.ts`, its test | A scan of a whole checkout for dashes in text written for a person; the testbed's `dashes` quick check owns it |
| `policy-reads.ts`, its test | A check that reads off the shared policy package exist; #301 retires that package, so nothing is left to check |
| `rust-reading.ts` | The Rust source parser the two scans above shared; nothing else uses it |
| `korean-in-code-ledger.json` | The old repository's ledger for the whole repository; this repository writes its own |
| `bunfig.toml` | Preloaded test helpers of other units; the new one preloads nothing |
| `Cargo.toml` | Inherited a workspace that is not this repository's; the new one stands alone |

## Data and policy files the old code read

| File | Read by | What becomes of it |
|---|---|---|
| The output gate's policy file, its `[faults]` table | `format` on both sides, for the reason of each fault the structure check finds | Not read; each fault has a fixed English reason, and the fault itself carries its tag and character as data |
| The English policy of a part | `english`, for the words read past and the Latin letters allowed per Hangul syllable | Not read; the consumer passes both in, with a default |
| The close and closing policy files | `english`'s tests only, as samples of a part's English policy | Not read; the tests hold sample values of their own |
| `WORDING.md`, the glossary of banned terms | `wording` on both sides, at a root the caller names | The caller names the path; the file name stays as the default the crate exposes |

## Ties to other units

| Tie | Used by | How it is cut |
|---|---|---|
| The policy crate's English struct | `english` | Plain parameters with a default |
| The policy crate's output gate table and fill helper | `format` | Fixed English reasons |
| The policy crate's close and closing tables | `english`'s tests | Inline sample values |
| `@bmo/policy`'s output gate file | `format.ts` | Fixed English reasons |
| `@bmo/util`'s fill helper | `format.ts` | Not needed once the reasons are fixed |
| `@bmo/util`'s count formatting | `answer-cap.ts` | A small helper of the package's own |
| `@bmo/util`'s checkout file list and `@bmo/roots`' source root | `wording.ts` and the dropped scans | The scans are dropped; `wording` takes its path from the caller |
| `@bmo/testkit`'s scratch folder | the dropped Korean scan's test | Dropped with it |
| The four test preloads | every test, through `bunfig.toml` | None of the kept tests needs them; the new `bunfig.toml` names none |

## Third-party dependencies

- The crate: `regex` 1.13.1, `unicode-segmentation` 1.13.3 and `url` 2.5.8, the versions the old lockfile
  resolved. `serde` is not needed once no policy struct is read.
- The package: `entities`, for HTML entities in a page read into lines. `marked` stays only if a kept module
  imports it. The Vue compilers, `typescript` as a runtime parser and the tree-sitter packages served only the
  dropped scans; `typescript` remains as the type checker.

## Tests outside `markup/` that exercised it

Eleven test files in the old repository imported the library. Two test the library's own functions alone
and are carried into `ts/tests/`; the other nine test their own unit's behaviour through it and stay with
that unit.

| Test | What it checks | Here |
|---|---|---|
| The repository check's wording test | The glossary: parsing, finding banned terms, the line written for them, allowed uses | Carried, minus its checkout-scan cases |
| The bot's answer-language test | Whether a text holds Korean | Carried |
| The bot's gate test | The structure check and the fold inside the output gate | Stays with the gate |
| The bot's Laya probes test | Markdown conversion inside the gate's probes | Stays with the gate |
| The bot's notify test | A notice's structure check and plain text | Stays with its unit |
| The bot's translator test | Halving a text for translation, over the archive | Stays with its unit |
| The bot's voice transcript test | The visible text of a transcript | Stays with its unit |
| The console's markdown test | The console's renderer, with the headings read here | Stays with the console |
| The voice tool's lines test | A type of the glossary, used by the voice tool | Stays with the tool |
| The tool host's two parity tests | The answer ceiling inside the tool host's answers | Stay with the tool host |
