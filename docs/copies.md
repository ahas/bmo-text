# The copies other repositories hold

Before this library stood, eleven bmo-next repositories copied or ported parts of the old `markup/` folder
into themselves. This document lists every such file, 82 across the eleven, says what each copies and how it
differs, and what each repository deletes and imports to depend on `bmo-markup` instead. No other repository
is changed by this one; the switch is each repository's own work, made against `docs/api.md`.

## How the copies were read

- **The set** was found by file name and by the definitions of the old modules' functions over every
  repository under bmo-next, and each candidate was measured by the lines it shares with the old modules.
- **Each copy was compared** with the old module it copies, blank lines ignored and the old module's inline
  tests set apart, and with the other repositories' copies of the same module.
- **Korean** was counted as lines holding Hangul and as escapes naming a Hangul code point.
- **The old modules' inline tests** are the baseline for a copied test file.

## What the copies have in common

- **The Rust modules were copied nearly whole.** The copies of `dashes`, `js`, `markdown`, `telegram` and
  `web_address` in gate, telegram, receiver and artifact match the old modules in their code; what differs is
  comments, an import path, the division's length made public, and the inline tests moved into `tests/`. This
  library's crate matches them in the same way.
- **The Hangul ranges are written as escapes in most copies.** Every Rust copy of `web_address` writes its
  Hangul ranges as `\x{…}` escapes, every copy of `wording` writes its Korean and Japanese ranges as `\u{…}`
  escapes, gate's `english` writes the syllable range as escapes, and the copied tests, gate's translation
  stages, receiver's `promises`, job's and agency's wording, the testbed's wording and Korean checks and the web
  client's `translation-parts` and `web-address` do the same. The work order's rule is that Korean is written
  plainly and counted in the ledger, never as an escape; this library writes all of them plainly. Each
  repository's switch removes its copies and with them most of these escapes; what remains in a repository's
  own code is that repository's to rewrite.
- **The structure check's fault reasons were cut two ways.** Telegram's copy of `format` returns fixed English
  sentences, the same ones this library returns. Gate's copy keeps reading them from a policy file of gate's
  own, in English sentences worded differently from telegram's. A consumer of this library gets the fixed
  sentences and the fault's kind and data, and words a reason of its own from those where it wants another.
- **The English test was cut two ways.** Gate's `english` takes a list of words it escapes and joins, longest
  first, where the old module and this library take one regular expression; gate's also adds a test of a
  markdown text's prose alone. Job holds the policy half, the struct of a part's two English values, with no
  test beside it. A consumer of this library passes its own pattern and weight.
- **The answer ceiling's cases were forked three ways.** The old cases file wrote its multi-byte samples in
  Hangul. Artifact's copy writes them in CJK ideographs, pocket's in Greek and accented Latin, and brain's keeps
  the Hangul but drops the cases of a single value. This library keeps the Hangul syllable for the repeated
  samples and English for the rest. The three behave alike under a count of code points; they differ only in
  which bytes they hold.
- **Several TypeScript-only modules were ported to Rust**: the answer ceiling (in artifact, brain, pocket,
  receiver and the testbed), the markdown structure and the translation parts (in gate and artifact), the
  finding reader and the HTML reading (in artifact) and the Hangul test of an answer (in receiver). This
  library's crate holds none of them yet; its package holds the TypeScript originals. Those Rust files stay
  in their repositories until the crate carries a twin of each.
- **The repository checks were ported to the testbed.** The old checkout scans for Korean in code, for dashes
  in text written for a person and for the glossary's banned terms are now the testbed's quick checks, ported
  to Rust with their own reading of files. They are the testbed's and stay there; this library supplies only
  the readings of one text.

## gate (22)

| File | Copies | How it differs | On switching |
|---|---|---|---|
| `crates/markup/src/dashes.rs` | the old `dashes` | Identical code; its doc comment names a rulebook in words instead of a path | Deleted; `bmo_markup::dashes` |
| `crates/markup/src/english.rs` | the old `english` | Takes a word list instead of a pattern, escaping and joining the words; adds a test of a markdown text's prose; its syllable range written as escapes | Deleted; `bmo_markup::english`, with gate building its pattern from its words and reading prose itself |
| `crates/markup/src/format.rs` | the old `format` | Reads its fault reasons and fill helper from gate's own policy crate, worded differently from telegram's | Deleted; `bmo_markup::format`, gate wording its reasons from the fault where it keeps its own |
| `crates/markup/src/js.rs` | the old `js` | Identical | Deleted; `bmo_markup::js` |
| `crates/markup/src/lib.rs` | the old `lib` | Adds the `text` module | Deleted with the crate |
| `crates/markup/src/markdown.rs` | the old `markdown` | Identical code but the division's length made public, as here | Deleted; `bmo_markup::markdown` |
| `crates/markup/src/telegram.rs` | the old `telegram` | Identical | Deleted; `bmo_markup::telegram` |
| `crates/markup/src/text.rs` | none | Gate's own readings: a grouped count, a number to so many places, the months a piece spans, fenced blocks and prose, the untranslated mark | Stays, as gate's own module |
| `crates/markup/src/web_address.rs` | the old `web_address` | Identical; its Hangul ranges as escapes | Deleted; `bmo_markup::web_address` |
| `crates/markup/src/wording.rs` | the old `wording` | Identical but its script ranges written as escapes | Deleted; `bmo_markup::wording` |
| `tests/markup/main.rs` | none | The test target holding the modules below | Deleted with them, or kept for `text` alone |
| `tests/markup/dashes.rs` | the old `dashes` tests | The 3 old cases | Deleted; the cases stand here |
| `tests/markup/english.rs` | the old `english` tests | 2 cases where the old had 3, on gate's word list, the syllable by escape | Deleted, or kept as gate's test of its own pattern |
| `tests/markup/format.rs` | the old `format` tests | The 3 old cases | Deleted |
| `tests/markup/js.rs` | the old `js` tests | The 4 old cases, the hash's Korean sample by escapes | Deleted |
| `tests/markup/markdown.rs` | the old `markdown` tests | The 7 old cases | Deleted |
| `tests/markup/telegram.rs` | the old `telegram` tests | The 4 old cases | Deleted |
| `tests/markup/text.rs` | none | Tests of gate's own `text` | Stays |
| `tests/markup/web_address.rs` | the old `web_address` tests | The 3 old cases, the Korean sample by escapes | Deleted |
| `tests/markup/wording.rs` | the old `wording` tests | The 4 old cases, the Korean glossary lines by escapes | Deleted |
| `crates/stages/src/markdown.rs` | the TS `markdown-structure` and `translation-parts`, ported | A Rust reading of headings, halves, sections and translation parts, with its own Korean test by escapes | Stays until the crate holds a twin of both |
| `tests/stages/markdown.rs` | the TS structure and translation tests, ported | Its Korean samples by escapes | Stays with the port |

Gate's workspace drops its `gate-markup` crate and depends on `bmo-markup` by path or git, keeping `text` as a
module of the crate that uses it; `testbed.toml` adds `bmo-markup` to its base libraries.

## telegram (10)

| File | Copies | How it differs | On switching |
|---|---|---|---|
| `dialect/src/format.rs` | the old `format` | Fixed English fault reasons, the same sentences as here | Deleted; `bmo_markup::format` |
| `dialect/src/js.rs` | the old `js` | The old module less its number reading, its byte-to-unit count and the hash | Deleted; `bmo_markup::js` |
| `dialect/src/markdown.rs` | the old `markdown` | Identical code but the division's length made public | Deleted; `bmo_markup::markdown` |
| `dialect/src/telegram.rs` | the old `telegram` | Identical | Deleted; `bmo_markup::telegram` |
| `dialect/src/web_address.rs` | the old `web_address` | Identical; its Hangul ranges as escapes | Deleted; `bmo_markup::web_address` |
| `dialect/tests/format.rs` | the old `format` tests | The 3 old cases | Deleted |
| `dialect/tests/js.rs` | the old `js` tests | 3 cases, the hash's left out with it | Deleted |
| `dialect/tests/markdown.rs` | the old `markdown` tests | The 7 old cases | Deleted |
| `dialect/tests/telegram.rs` | the old `telegram` tests | The 4 old cases | Deleted |
| `dialect/tests/web_address.rs` | the old `web_address` tests | The 3 old cases, the Korean sample by escapes | Deleted |

The dialect keeps its own declaration and its contract test, and imports the readings from `bmo-markup`.

## receiver (11)

| File | Copies | How it differs | On switching |
|---|---|---|---|
| `src/dashes.rs` | the old `dashes` | Identical code; its doc comment drops the rulebook's path | Deleted; `bmo_markup::dashes` |
| `src/web_address.rs` | the old `web_address` | Identical; its Hangul ranges as escapes | Deleted; `bmo_markup::web_address` |
| `src/wording.rs` | the old `wording` | The old module less the line hits and the finding of every use in a file; its script ranges as escapes | Deleted; `bmo_markup::wording` |
| `src/text.rs` | part of the old `js` | A length in UTF-16 units and a grouped count, as the console writes a bound | The length becomes `bmo_markup::js::units`; the grouped count stays the receiver's |
| `crates/room-calls/src/js.rs` | the old `js` | Identical | Deleted; `bmo_markup::js` |
| `crates/room-calls/src/reading.rs` | the TS `answer-cap`, ported | The ceiling and its wording beside the reading tool's answer | The ceiling stays until the crate holds a twin; the answer is the receiver's own |
| `crates/room-calls/src/promises.rs` | the TS `answer-language`, ported, among the receiver's own | A Hangul test of a text by escapes, inside the receiver's promise reading | Stays until the crate holds a twin |
| `tests/dashes.rs` | the old `dashes` tests | The 3 old cases | Deleted |
| `tests/js.rs` | the old `js` tests | The 4 old cases, the hash's Korean sample by escapes | Deleted |
| `tests/web_address.rs` | the old `web_address` tests | The 3 old cases, the Korean sample by escapes | Deleted |
| `crates/room-calls/tests/reading.rs` | the TS `answer-cap` tests, in part | The reading tool's answer under the ceiling | Stays with the port |

## artifact (11)

| File | Copies | How it differs | On switching |
|---|---|---|---|
| `src/helpers/js.rs` | part of the old `js` | White space, trim and characters, with a leading integer's reading and an address encoding set of its own | The copied part becomes `bmo_markup::js`; the integer reading and the encoding set stay artifact's |
| `src/helpers/web_address.rs` | the old `web_address` | Identical but the import path; its Hangul ranges as escapes | Deleted; `bmo_markup::web_address` |
| `src/helpers/ceiling.rs` | the TS `answer-cap`, ported | The ceiling of a list and of one value, with its wording | Stays until the crate holds a twin |
| `src/markup/finding_markdown.rs` | the TS `finding-markdown`, ported | The finding drawn with the words of artifact's language pack, a part called by its key where it has none; the design this library's package follows | Stays until the crate holds a twin |
| `src/markup/markdown_structure.rs` | the TS `markdown-structure`, ported | Headings, marks and sections read the same way | Stays until the crate holds a twin |
| `src/verbs/archive_docs.rs` | the TS `html-reading`, ported, inside artifact's document verbs | The page reading and its pictures | The reading stays until the crate holds a twin; the verbs are artifact's own |
| `tests/helpers.rs` | the old `js` and `web_address` tests and the ceiling's | 20 cases over the helpers and the Rust twins | The copied modules' cases are deleted; the twins' stay |
| `tests/markup.rs` | the TS structure and finding tests, ported | English samples, the old sample names of real places carried over translated into English | Stays with the ports |
| `tests/html_read.rs` | the TS `html-reading` tests, ported, through artifact's verb | The verb's run of a page | Stays, as the verb's test |
| `tests/cases/answer-cap-cases.json` | the old cases file | Multi-byte samples in CJK ideographs | Stays with the ceiling's twin, or is replaced by this library's file once the crate holds the ceiling |
| `client/app/domain/web-address.ts` | part of the TS `web-address` | Where a bare address ends and whether an address is a web one, its Hangul ranges as escapes | Deleted; `bmo-markup/web-address` |

## brain (5)

| File | Copies | How it differs | On switching |
|---|---|---|---|
| `crates/brain-support/src/js.rs` | the old `js` | Identical but a safe-integer bound added | Deleted; `bmo_markup::js`, the bound kept as brain's own constant |
| `crates/brain-support/src/ceiling.rs` | the TS `answer-cap`, ported | The ceiling of a list alone, with its wording | Stays until the crate holds a twin |
| `crates/brain-support/tests/js.rs` | the old `js` tests | 5 cases, the hash's Korean sample by escapes | Deleted |
| `crates/brain-support/tests/ceiling.rs` | the TS `answer-cap` tests, ported | Over its fixture | Stays with the ceiling |
| `crates/brain-support/tests/fixtures/answer-cap-cases.json` | the old cases file | The Hangul samples kept, the cases of a single value dropped | Stays with the ceiling |

## pocket (3)

| File | Copies | How it differs | On switching |
|---|---|---|---|
| `sdk/contract/src/ceiling.rs` | the TS `answer-cap`, ported | The ceiling of a list and of one value, for a tool's answer | Stays until the crate holds a twin |
| `sdk/contract/tests/contract/ceiling.rs` | the TS `answer-cap` tests, ported | Over its fixture | Stays with the ceiling |
| `sdk/contract/tests/fixtures/answer-cap-cases.json` | the old cases file | Multi-byte samples in Greek and accented Latin | Stays with the ceiling |

## job (4)

| File | Copies | How it differs | On switching |
|---|---|---|---|
| `crates/queue-rows/src/js.rs` | part of the old `js` | White space, trim and lengths, with a cut and a leading integer of its own | The copied part becomes `bmo_markup::js`; the cut and the integer stay job's |
| `crates/queue-rows/src/wording.rs` | part of the old `wording` | The banned terms read out of the glossary its settings name and found in a text outside quotes; its script ranges as escapes | The reading becomes `bmo_markup::wording`'s glossary reading and banned terms; where the glossary stands stays job's setting |
| `crates/queue-rows/tests/wording.rs` | part of the old `wording` tests | Over job's settings | Kept for job's settings, the reading's cases deleted |
| `crates/policy/src/english.rs` | the old policy crate's English struct | The two values of a part's English policy, with no test | Stays as job's policy; job passes the two values to `bmo_markup::english` |

## agency (3)

| File | Copies | How it differs | On switching |
|---|---|---|---|
| `crates/server/src/wording.rs` | the old `wording` | The old module less the finding of every use in a file, its length read through agency's own `text`; its script ranges as escapes | Deleted; `bmo_markup::wording` |
| `crates/server/src/text.rs` | part of the old `js` | Length, white space, trim and the characters within a bound, and a grouped count | The copied part becomes `bmo_markup::js`; the grouped count stays agency's |
| `tests/server/wording.rs` | part of the old `wording` tests | 5 cases, a Korean sample by escapes | Deleted |

## node (2)

| File | Copies | How it differs | On switching |
|---|---|---|---|
| `crates/hardware/src/js.rs` | the old `js` number reading, extended | A figure read and written as JavaScript reads and writes it: parsing, rounding and showing a number | The number reading may become `bmo_markup::js::number`; the rest is node's own and stays |
| `tests/hardware/js.rs` | none of the old cases | Tests of node's number readings | Stays |

## testbed (7)

| File | Copies | How it differs | On switching |
|---|---|---|---|
| `suite/src/quick/rewrite.rs` | the old `dashes`, reshaped | The dash rewrite over a markdown file's prose, read by a markdown parser, a piece at a time | Stays, as the testbed's check; it may take `bmo_markup::dashes` for the rewrite of one piece |
| `suite/src/quick/wording.rs` | the TS `wording` check, ported | The glossary held to every file a declaration names, allowed uses included, over a checkout; its script ranges as escapes | Stays, as the testbed's check; it may take `bmo_markup::wording` for reading the glossary and finding a term |
| `suite/src/quick/korean.rs` | the TS Korean scan, ported | Korean lines per code file held to a ledger, read by kind | Stays, as the testbed's check |
| `suite/src/quick/dashes.rs` | the TS written-dash scan, ported | Dashes in the literals of the files a declaration names as text for a person | Stays, as the testbed's check |
| `suite/tests/integration/quick_wording.rs` | the TS wording tests, ported | 9 cases, the Korean samples by escapes | Stays with the check |
| `core/src/answer.rs` | the TS `answer-cap`, ported | The ceiling of a list, with its wording | Stays until the crate holds a twin |
| `core/tests/integration/answer.rs` | the TS `answer-cap` tests, ported | 5 cases | Stays with the ceiling |

## web (4)

| File | Copies | How it differs | On switching |
|---|---|---|---|
| `client/apps/web/shared/markdown-structure.ts` | the TS `markdown-structure` | Identical | Deleted; `bmo-markup/markdown-structure` |
| `client/apps/web/shared/translation-parts.ts` | the TS `translation-parts` | Identical but its syllable range written as escapes | Deleted; `bmo-markup/translation-parts` |
| `client/apps/web/shared/web-address.ts` | the TS `web-address` | Identical; its Hangul ranges as escapes, as the old module wrote them | Deleted; `bmo-markup/web-address` |
| `client/tests/app/domain/markdown.test.ts` | the old console renderer's test, in part | 29 cases where the old renderer test had 59, reading headings through the shared copy | Stays, as the renderer's test, its headings imported from `bmo-markup/markdown-structure` |

The client adds `bmo-markup` as a dependency and imports the three modules by name in place of its shared
copies.

## Files that are not copies

- **cache-warmer and kakaotalk** measure text with the standard library's UTF-16 encoding alone; neither holds
  any of the old modules.
- **base/ui's `shared/wording.ts`** is the type of the console's translations, not the banned-term check.
- **artifact's `src/wording.rs` and web's `server/src/wording.rs`** read a language pack's documents; they
  share only a file name with the glossary check.
- **The renderers in the artifact and web clients** (artifact's `finding-markdown.ts`, web's `markdown.ts`)
  turn markdown into HTML in the browser with marked and DOMPurify; they read the structure this library's
  `finding-markdown` and `markdown-structure` write, and are not copies of them.
- **The fill helpers in eight repositories** copy the old policy crate's template helper, not a markup
  module; this library has no such helper, since none of its readings fills a template.
