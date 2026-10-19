# Design

`bmo-markup` is a base library: the text readings that the chat services, the console, the archive, the
gate and the testbed would otherwise each hold a copy of. It holds the mechanism of a reading and a safe
default for anything that differs per consumer; the consumer supplies the rest. It knows no consumer by
name and depends on no repository.

## Layout

- **The crate at the root, the package beside it.** The crate `bmo-markup` is the repository's root
  project: its manifest and lockfile are the only manifest and lockfile at the root, with `src/` and
  `tests/`. The package `bmo-markup` stands in `package/` with its own manifest and lockfile, a project of
  its own in the testbed's sense, so the two build tools' files never mix. No folder is named for a
  language.
- **One crate, split into modules.** Nothing in the crate has to stand apart, so there is no `crates/`
  workspace. Each reading is one module under `src/`; its tests are one file under `tests/`, beside the
  repository test that holds the repository's own shape.
- **One package, one file per module.** Each module is a file at the top of `package/`, exported by its
  name; its tests are under `package/tests/` with their data files. The package's `check` script runs its
  static check and its tests together, and is how the testbed checks and tests it.
- **The package is named `bmo-markup`,** as the crate is and as `bmo-ui` is named and imported, so a
  consumer imports a module as `bmo-markup/<module>`.
- **One version.** `VERSION` holds the repository's semver; the crate and the package carry the same
  number. A change to a public signature or to what a reading answers for a text is a major change.

## The twins

The crate and the package read a text the same way: lengths are counted in UTF-16 units, as Telegram and
JavaScript count them, and every space, trim and character class is JavaScript's. A module present in both
gives one answer for one text. Some readings exist on one side only, because only that side's consumers
need them; `docs/api.md` names which. There is no test that runs one side against the other: each side's
tests hold its own cases, and a case added to one is added to the other by hand.

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
  neutral English. A test file keeping Korean is counted in the ledger like any other file.
- **No name of a person, an agent or a real place** stands in a test; the example text uses neutral words.

## What the library does not hold

- **Checks over a whole checkout.** Counting the Korean lines of every code file against a ledger, finding
  dashes in the text a repository writes for a person, and holding every file to a glossary are the
  testbed's quick checks, run over the checkout it is given. This library supplies the readings of one text
  that such a check can use; it does not walk a repository.
- **A check of reads off a shared policy package.** No shared policy package exists; each unit owns its
  policy files.
- **Any server, deployment, data folder or table.**
