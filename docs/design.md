# Design

`bmo-text` is a base library: the text readings that the chat services, the console, the archive, the
gate and the testbed would otherwise each hold a copy of. It holds the mechanism of a reading and a safe
default for anything that differs per consumer; the consumer supplies the rest. It knows no consumer by
name and depends on no repository.

## Layout

- **The crate at the root, the package beside it.** The crate `bmo-text` is the repository's root
  project: its manifest and lockfile are the only manifest and lockfile at the root, with `src/` and
  `tests/`. The package `bmo-text` stands in `package/` with its own manifest and lockfile, a project of
  its own in the testbed's sense, so the two build tools' files never mix. No folder is named for a
  language.
- **One crate, split into modules.** Nothing in the crate has to stand apart, so there is no `crates/`
  workspace. Each reading is one module under `src/`; its tests are one file under `tests/`, beside the
  repository test that holds the repository's own shape.
- **One package, one file per module.** Each module is a file at the top of `package/`, exported by its
  name; its tests are under `package/tests/`. The package's `check` script runs its static check and its
  tests together, and is how the testbed checks and tests it.
- **The cases at the root.** The cases both sides are held to stand in `cases/`, one JSON file per module,
  read by the tests of both projects (「The twins」).
- **The package is named `bmo-text`,** as the crate is and as `bmo-ui` is named and imported, so a
  consumer imports a module as `bmo-text/<module>`.
- **One version.** `VERSION` holds the repository's semver; the crate and the package carry the same
  number. The public API stated in `docs/api.md` is fixed at major version 2; a change to a public
  signature or to what a reading answers for an input is a major change.

## The twins

- **One list of modules.** The crate and the package carry the same readings under the same names, a module
  being `snake_case` in the crate and `kebab-case` in the package: `answer_cap`, `answer_language`, `dashes`,
  `english`, `finding_markdown`, `format`, `html_reading`, `markdown`, `markdown_structure`, `telegram`,
  `translation_parts`, `web_address` and `wording`. The crate's `js` is the one module apart: it is how the
  crate counts, spaces and trims as JavaScript does, which the package has from its language, so it is the
  mechanism of the twinning rather than a reading.
- **One answer for one input.** A module gives the same answer on both sides for the same input. Spaces,
  trims and character classes are JavaScript's on both sides, and a regular expression the package writes
  is read the same way by the crate.
- **A length says what it counts.** A length is counted in UTF-16 units, as Telegram and JavaScript count
  it, unless its name says `chars`: such a length counts characters as a reader does, one to a code point,
  so a Hangul syllable and an emoji are one each. A budget of estimated tokens is named for tokens. An
  index into a string is in UTF-16 units too; a crate function that reads a run of characters rather than a
  string indexes that run, and names the same places.
- **The cases are shared.** Each module is held to one file of cases under `cases/`, named as the package
  names the module, and both sides' tests read every case in it: the inputs of each public function and
  the answer expected, in the package's JSON shape (camelCase keys, `null` for none). A case is added once,
  to that file, and a side that answers it differently fails its own tests. Each side's own tests stand
  beside the shared ones for what only that side has, such as a type's shape or a closure the caller
  passes.
- **Items may differ where a side's consumers differ.** Within one module, a side may carry an item the
  other does not, because only that side's consumers need it; `docs/api.md` names each such item.

## What the consumer decides

- **The words a reader sees.** A search finding's structure lines name their parts by an English key
  (`VERDICT`, `SUMMARY`, `ADDRESS` and the rest). The finding reader takes the words a reader sees for each
  key from the consumer, which reads them from its own language pack; where it passes none, the key itself
  is written. The library holds no human-language label.
- **A part's English policy.** Whether a text is written in English is read against the words the part reads
  past (a regular expression, matched as the consumer writes it) and how many Latin letters a Hangul
  syllable is weighed against. The consumer passes both, from its own policy; the default reads past nothing
  and weighs at zero, so any Hangul left fails the test.
- **The glossary's path.** The banned-term check reads the glossary at the path the caller names. The
  library exposes the conventional file name, `WORDING.md`, as a default the caller may use.

## What the library fixes itself

- **The structure check's reasons.** A fault the structure check finds in a piece of markup carries its kind
  and its data (the tag, the character, the tag still open, the tag it may not stand in) and one fixed
  English sentence saying what is wrong. A consumer that shows a reason in another language translates from
  the kind and the data, never from the sentence.
- **Telegram's limits.** The text and caption limits are Telegram's, not a policy, and stand as constants.

## Korean

- **The library reads Korean, so some of its code holds Hangul.** A Hangul-aware reading (whether a line is
  Korean, whether a letter glued to an address ends it, whether a text is English enough) names the Hangul
  ranges it reads. These are written as literal characters, never as escape sequences or char codes, and
  every file holding them is counted in `korean-ledger.json`.
- **A test keeps Korean only where English would stop it testing what it tests:** script detection, UTF-16
  widths of Hangul, where an address ends against a Hangul letter, Korean punctuation and sentence ends, the
  Korean lines of a glossary, and a ceiling counting Hangul as characters. Every other example text is
  neutral English. A test file keeping Korean is counted in the ledger like any other file; a cases file
  under `cases/` is data rather than code, keeps Korean by the same rule, and is not counted.
- **No name of a person, an agent or a real place** stands in a test; the example text uses neutral words.

## What the library does not hold

- **Checks over a whole checkout.** Counting the Korean lines of every code file against a ledger, finding
  dashes in the text a repository writes for a person, and holding every file to a glossary are the
  testbed's quick checks, run over the checkout it is given. This library supplies the readings of one text
  that such a check can use; it does not walk a repository.
- **A check of reads off a shared policy package.** No shared policy package exists; each unit owns its
  policy files.
- **Any server, deployment, data folder or table.**
