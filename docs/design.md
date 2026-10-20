# Design

`bmo-text` is a base library: the text readings that the chat services, the console, the archive, the
gate and the testbed would otherwise each hold a copy of, and the MCP server through which a coding
harness calls the pure ones. It holds the mechanism of a reading and a safe default for anything that
differs per consumer; the consumer supplies the rest. It knows no consumer by name and depends on no
repository.

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
  name, that answers by calling the crate's WebAssembly build (「One implementation」); its tests are under
  `package/tests/`. The package's `check` script runs its static check and its tests together, and is how
  the testbed checks and tests it; its `build` script makes the WebAssembly build.
- **The cases at the root.** The cases both sides are held to stand in `cases/`, one JSON file per module,
  read by the tests of both projects (「The twins」).
- **The package is named `bmo-text`,** as the crate is and as `bmo-ui` is named and imported, so a
  consumer imports a module as `bmo-text/<module>`.
- **The MCP server is the crate's binary.** `bmo-text`, built from `src/bin/`, starts the server with
  `bmo-text mcp` (「The MCP server」). It and the crate's `mcp` module are compiled with the crate's `mcp`
  feature, which is on by default so the crate's own tests reach the server; the WebAssembly build, and a
  Rust consumer taking the readings alone, leave it out with `default-features = false`.
- **The harness plugin in `plugin/`.** The plugin `bmo-text` is the manifest a harness installs, the MCP
  server it starts, and the `bmo-text` skill telling the harness when to call which tool.
- **One version.** `VERSION` holds the repository's semver; the crate, the package and the plugin carry
  the same number. Until the first release the number is `0.1.0` with a `-rev.N` suffix, N counting the
  versions cut so far (`docs/api.md`, 「Version numbers」). The public API stated in `docs/api.md`, the MCP
  server's tools among it, is fixed at the revision `VERSION` names; a change to a public signature, to a
  tool's input or output, or to what a reading answers for an input cuts the next revision.

## One implementation

- **The readings are written once, in the crate.** The package holds no reading of its own: it is the crate
  compiled to WebAssembly, and each of its modules is a thin file that hands its arguments to the build and
  gives back what the build answers, typed as the package's API states it.
- **The binding is the crate's `wasm` module,** compiled only with the crate's `wasm` feature, so a Rust
  consumer never builds it. It takes the package's arguments, calls the crate's public functions and writes
  each answer in the package's shape: camelCase keys, `null` for none, a field the package leaves out when
  empty left out, and every index and length counted in UTF-16 units. It holds no reading either; a value it
  cannot express in the package's shape is a fault of the binding, not a second answer.
- **The package keeps only what a JavaScript caller holds.** A module's file does in JavaScript what only
  JavaScript can: reading a file the caller names, calling back a closure the caller passes, handing back the
  caller's own objects (the items a cut keeps, the terms a glossary lists) rather than copies of them, and
  exporting the two block shapes of a drawn finding as JavaScript regular expressions.
- **The build is carried in the package.** `package/wasm/` holds what the build made: the binding's
  JavaScript glue and its types, the module's bytes written into a JavaScript module, and the hash of the
  crate's sources it was made from. It is committed, so a consumer taking the package needs neither Rust nor
  the build's tools. The package's tests fail where the crate's sources no longer hash to what the build was
  made from, and `bun run build` in `package/` makes it again.
- **The module is instantiated on import, synchronously.** The first module a consumer imports compiles and
  instantiates the bytes the package carries, before anything is called, so no initialisation is awaited and
  the package loads alike in Bun, in Node and in a browser bundle such as a Nuxt client's, with no bundler
  plugin for WebAssembly. Every module calls through the instantiated build as a value it imports, so a
  bundler that takes the package for free of side effects, as a Nuxt server's does, still instantiates it
  before the first call. A browser compiles a module synchronously on its main thread only up to a size it
  sets, which the build is kept well under.
- **The build's tools** are the Rust target `wasm32-unknown-unknown`, the `wasm-bindgen` command at the
  version the crate's `wasm-bindgen` dependency is locked to, and Binaryen's `wasm-opt` from the package's
  development dependencies. The crate is compiled with its `wasm` feature alone, its default features off, under
  its `wasm` profile, which optimises for size.

## The twins

- **One list of modules.** The crate and the package carry the same readings under the same names, a module
  being `snake_case` in the crate and `kebab-case` in the package: `answer_cap`, `answer_language`, `dashes`,
  `english`, `finding_markdown`, `format`, `html_reading`, `markdown`, `markdown_structure`,
  `translation_parts`, `web_address` and `wording`. The crate's `js` is the one module apart: it is how the
  crate counts, spaces and trims as JavaScript does, which the package has from its language, so it is the
  mechanism of the twinning rather than a reading. The crate's `wasm` and `mcp` are bindings, not readings:
  each hands a caller's arguments to the readings and gives back their answers in its caller's shape.
- **One answer for one input.** A module gives the same answer on both sides for the same input, the
  package's answer being the crate's. Spaces, trims and character classes are JavaScript's on both sides,
  and a regular expression a caller writes is read by the crate. A text crosses into the build as UTF-8, so
  an unpaired surrogate in a string the package is given reaches the crate as U+FFFD.
- **A length says what it counts.** A length is counted in UTF-16 units, as Telegram and JavaScript count
  it, unless its name says `chars`: such a length counts characters as a reader does, one to a code point,
  so a Hangul syllable and an emoji are one each. A budget of estimated tokens is named for tokens. An
  index into a string is in UTF-16 units too; a crate function that reads a run of characters rather than a
  string indexes that run, and names the same places.
- **The cases are shared.** Each module is held to one file of cases under `cases/`, named as the package
  names the module, and both sides' tests read every case in it: the inputs of each public function and
  the answer expected, in the package's JSON shape (camelCase keys, `null` for none). A case is added once,
  to that file, and a side that answers it differently fails its own tests: the crate's tests read the
  crate natively, and the package's read it through the WebAssembly build and its binding. Each side's own
  tests stand beside the shared ones for what only that side has, such as a type's shape or a closure the
  caller passes.
- **Items may differ where a side's consumers differ.** Within one module, a side may carry an item the
  other does not, because only that side's consumers need it; `docs/api.md` names each such item.

## The MCP server

- **A harness's first entry point.** A coding harness given the `bmo-text` plugin starts `bmo-text mcp`,
  which speaks MCP over its standard input and output, and calls its tools to check or reshape text it is
  about to send or file. The server keeps nothing between calls, calls no other unit, and knows nothing of
  who started it: no tool, field or answer names a caller.
- **Seven tools, each a thin call of one reading.** A tool takes its input, calls one public function of
  the crate and writes that function's answer as its structured output; it holds no reading of its own.
  `text_dashes` calls `dashes::read_dashes`, `text_language` calls `answer_language::text_language`,
  `text_markdown_structure` calls `markdown_structure::markdown_structure`, `text_html_reading` calls
  `html_reading::read_html_page`, `text_translation_parts` calls `translation_parts::translation_parts`,
  `text_web_addresses` calls `web_address::web_addresses_in` and `text_wording` calls `wording::text_swaps`.
  `docs/api.md` states each tool's input and output.
- **Its fields are the units' wire format:** camelCase keys, and an optional input left out or `null` alike.
  A line number counts from one; an index into a list counts from nought.
- **The caller decides what is its own.** `text_wording` reads the glossary in the folder the caller names,
  the server's working directory where it names none, which is the project the harness started it in.
  `text_translation_parts` takes its token budget from the caller, the library's default where it names
  none; `text_dashes` reads a wrapped document only when the caller says the text is one.
- **An answer stays under the ceiling.** The HTML reading is the one tool whose answer grows with its
  input, so it answers the lines from the one the caller names, as many as fit under the answer ceiling
  (`answer_cap`), and says whether more are left. The pictures a page carries inside itself are named in
  its lines and not handed over.
- **A tool that cannot answer says so.** A glossary that cannot be read or does not parse is the tool's
  error, one English sentence, never an empty answer read as a clean text.

## What the consumer decides

- **The words a reader sees.** A search finding's structure lines name their parts by an English key
  (`VERDICT`, `SUMMARY`, `ADDRESS` and the rest). The finding reader takes the words a reader sees for each
  key from the consumer, which reads them from its own language pack; where it passes none, the key itself
  is written. The library holds no human-language label.
- **A part's English policy.** Whether a text is written in English is read against the words the part reads
  past (a regular expression, matched as the consumer writes it) and how many Latin letters a Hangul
  syllable is weighed against. The consumer passes both, from its own policy; the default reads past nothing
  and weighs at zero, so any Hangul left fails the test.
- **Where a code text's comments and literals stand.** The judgement of letters that are not English places
  each letter it finds by the comments and literals the consumer's own reader of the code language marks.
  The library reads no code language; with no stretch named, every letter is in the code.
- **The glossary's path.** The banned-term check reads the glossary at the path the caller names. The
  library exposes the conventional file name, `WORDING.md`, as a default the caller may use.

## What the library fixes itself

- **The structure check's reasons.** A fault the structure check finds in a piece of markup carries its kind
  and its data (the tag, the character, the tag still open, the tag it may not stand in) and one fixed
  English sentence saying what is wrong. A consumer that shows a reason in another language translates from
  the kind and the data, never from the sentence.
- **What is not English.** Whether a text is not English is a question of script, not of any one language:
  a letter of any script but Latin is not English, written as itself or named by one of the escapes code
  languages share. Neither the scripts nor the escape shapes are a policy, so neither is passed in.
- **No platform's limits.** A division into parts takes its limit from the caller, and the library names
  no chat platform's text or caption limit, bot token or file size: those are the platform's own, stated
  where that platform is served.

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

## Personal data

- **No file and no commit carries personal data.** Names, handles, account ids, email and network
  addresses, host names, home paths, devices, company names, a room member's display name or id, and a
  line of a real conversation are kept out of code, comments, documents, test fixtures and commit
  messages. A value that differs per deployment is configuration the consumer passes in; an example is a
  neutral word. `docs/PERSONAL-DATA.md` states the rule and the two values the owner allows.
- **The repository test holds every file to it.** A file fails the test where it holds an email address,
  a home path, a phone number, a token or a private address. The report names the file, the line and the
  kind, never the value. The encoded bytes the WebAssembly build writes are not scanned: they are
  generated, and a long run of encoded bytes is no one's writing.

## What the library does not hold

- **Checks over a whole checkout.** Counting the Korean lines of every code file against a ledger, finding
  dashes in the text a repository writes for a person, and holding every file to a glossary are the
  testbed's quick checks, run over the checkout it is given. This library supplies the readings of one text
  that such a check can use; it does not walk a repository.
- **A check of reads off a shared policy package.** No shared policy package exists; each unit owns its
  policy files.
- **Any HTTP server, deployment, data folder or table.** The MCP server answers the one harness that started
  it, over its standard input and output, and keeps nothing.
