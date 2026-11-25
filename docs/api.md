# Public API

Everything a consumer may use, project by project and module by module, and the tools of the MCP server,
fixed at the release the commit is tagged with (「Version numbers」). An item not named here is internal and
may change in any version. Both sides carry the same modules (`docs/design.md`, 「The twins」), the crate's
`js` apart, and a module gives one answer for one input on either side. Lengths are UTF-16 units and spaces are
JavaScript's white space; a length named `chars` counts code points. An item one side carries and the other
does not is marked as such.

## The Rust crate `bmo-text`

A function of the package is named in `snake_case` here and a constant in `SCREAMING_SNAKE_CASE`; a
function that answers none in the package answers an `Option`, and a record is a struct of the same
fields. Where an entry calls a module "the package's", the package's entry says what it does.

### `answer_cap`

- **`MAX_ANSWER_CHARS`**, **`MAX_VALUE_CHARS`**, **`Capped`**, **`cap_items`** and **`cap_value`** are the
  package's `answer-cap`. **`Said`** is what `cap_items` is told: the unit counted, how to ask for the rest,
  and the ceiling, which defaults to `MAX_ANSWER_CHARS`. The render is a closure given each prefix tried.

### `answer_language`

- **`holds_korean`** is the package's `holdsKorean`.
- **`TextLanguage`** and **`text_language`** are the package's `TextLanguage` and `textLanguage`, a language
  being `Ko` or `En` and its `code` the package's `ko` or `en`.

### `dashes`

- **`without_dashes`** takes a text and whether it is wrapped, and gives the text with every dash left between
  words replaced the way a person types the sentence: a name and its explanation joined by a colon, an aside
  between two dashes set off by commas (or brackets where it has commas of its own), any further dash of the
  sentence a colon or a comma, an unspaced dash between digits a range. What is not prose keeps its dashes:
  code spans and blocks, quotations, quoted lines, table rows, addresses, a dash opening or ending a line. A
  wrapped text reads a paragraph or a list item as one piece across its line breaks; an unwrapped one reads
  each line alone, as a chat answer is written.
- **`read_dashes`** reads a text the same way and gives a **`DashReading`**: its `text`, the one
  `without_dashes` gives, and how many dashes it `found` and replaced, nought for a text that keeps its own.

### `english`

- **`EnglishTest`** is the test of whether a text is English, as a part's policy declares it.
  - **`try_new`** takes the set-aside pattern, a regular expression of the words read past wherever they
    stand, lookaround and backreferences included, and the number of Latin letters each Hangul syllable left
    is weighed against; it gives the compiler's message for a pattern that does not compile. **`new`** is
    the same, and panics on such a pattern.
  - **`passes`** answers whether a text is English: with a weight of zero, no Hangul syllable may be left
    once the set-aside words are read past; otherwise the text holds some Latin letters and fewer syllables
    than one to every so many of them.
  - **`Default`** reads nothing past and weighs at zero, so any Hangul syllable fails it.
- **`holds_non_english`** answers whether a text holds a letter that is not English, whatever language it
  is: a letter of any script but Latin (Hangul, kana, Han, Cyrillic, Greek, Arabic and every other), written
  as itself or named by an escape. The escapes read are the four shapes code languages share, their code
  point in hexadecimal: a braced `\u{…}`, a four-digit `\u`, an eight-digit `\U` and a pattern's braced
  `\x{…}`; two four-digit escapes naming a surrogate pair name the one letter the pair stands for. A text
  may be any code language, a comment as much as a literal, so an escape is read wherever it stands and
  whatever backslashes come before it, as a pattern written in a literal doubles its backslash and still
  names the letter. A Latin letter with a mark, a digit, a symbol, and punctuation of any script are no
  such letter, nor is a letter of no script of its own such as the micro sign.
- **`non_english_letters`** gives each such letter of a text in the order written, placed by the stretches
  the caller's own reader of the code language marks. **`NonEnglishLetter`** is one: its `start` and `end`,
  covering the letter or the whole escape naming it, the `letter`, whether it was `escaped`, and its
  `place`. **`LetterPlace`** is that place, by where the letter's start falls: in one of the comments, in
  one of the literals, or else in the code itself; its `code` is the package's name for it, `comment`,
  `literal` or `code`. **`CodeStretches`** are the `comments` and the `literals` the caller names, each a
  range; its `Default` names none, which places every letter in the code. Every index is in UTF-16 units.

### `finding_markdown`

- **`PART_KEYS`**, **`FindingWords`**, **`finding_markdown`**, **`footnote_marks`**, **`picture_caption`**,
  **`FINDING_BLOCK`**, **`FINDING_MAP`** and **`finding_excerpt`** are the package's `finding-markdown`; the
  package's `englishFindingWords` is `FindingWords`' `Default`, and the two block shapes are compiled
  regular expressions.

### `format`

- **`escape_html`** escapes the three characters Telegram's HTML mode reads outside a tag, the ampersand
  first; **`escape_attribute`** escapes the double quote as well.
- **`is_linkable`** answers whether an address is one Telegram takes in a link: http or https, no space.
- **`visible_text`** is what a reader sees of a piece of markup: the tags taken out and the three escapes put
  back.
- **`expandable_markup`** wraps markup already built in the expandable quote a fold is; **`expandable`**
  escapes a text first.
- **`LONG_TEXT_LENGTH`** is 600: past it a message is long enough for a fold to help.
- **`MarkupFault`** is what the structure check finds: an unescaped character, an unsupported tag, a tag never
  closed, a closing tag that was never opened (with the tag still open where there is one), or a tag nested
  where Telegram refuses it (with the tag it stands in).
- **`markup_fault`** gives the first fault of a piece of markup, or none: the tags paired and nested as
  Telegram takes them, and the characters between them read for whether the escaping held.
- **`fault_reason`** says a fault in one fixed English sentence naming its tag or character. A consumer
  showing it in another language translates from the fault's kind and data, not from the sentence.

### `html_reading`

- **`read_html`**, **`HtmlReading`** and **`HtmlPicture`** are the package's `html-reading`; a picture's media
  type is its `media_type`.
- **`read_html_page`** and **`HtmlPage`** are the crate's own: a page's `title` and as many of its `lines`,
  from the index the caller names, as fit under `answer_cap`'s `MAX_ANSWER_CHARS` joined a line apart, and
  whether `more` are left after them. A line over the ceiling on its own is given alone, cut at the ceiling,
  so every page moves the reading on. The pictures are named after the stem given, as `read_html` names
  them.

### `js`

The crate's own: JavaScript's counting, spaces and trims, which the package has from its language.

- **`SPACE`** and **`DOT`** are JavaScript's `\s` and `.` as regular expression class bodies.
- **`is_space`**, **`trim`** and **`trim_end`** are JavaScript's white space and its trims.
- **`number`** reads a short text as JavaScript's `Number` would, a blank text being nought and anything not a
  finite number none.
- **`units`** and **`units_of`** are a text's and a run of characters' length in UTF-16 units; **`within`** is
  how many characters fit in a number of units; **`units_before`** turns a byte index a regular expression
  found into a UTF-16 count.
- **`chars`** and **`text`** turn a text into its characters and back; **`find`** and **`rfind`** find a run of
  characters at or after, or at or before, an index.
- **`bun_hash`** is what Bun's hash answers for a text: Wyhash of its UTF-8 bytes with seed 0.
- **`encode_uri_component`** and **`decode_uri_component`** are JavaScript's, the decoding none where
  JavaScript's would throw: an escape that is not two hexadecimal digits, or escaped bytes that are not UTF-8.

### `wasm`

The crate's own, compiled only with the `wasm` feature: the binding the package's WebAssembly build exports
(`docs/design.md`, 「One implementation」). Its exports are what the package's modules call and are no API of
their own; a Rust consumer has no use for it.

### `markdown`

The markdown an agent writes, read line for line into Telegram's HTML and plain text.

- **`Pair`**, **`Inline`**, **`Line`**, **`Block`** and **`Document`** are the reading: a document is its blocks,
  each with the line breaks that stood before it; a block is a run of lines, a code block with its
  language, or a quote holding a document of its own; a line is its inline pieces, which are words, code spans, formatting pairs (bold,
  italic, strike, spoiler) and links.
- **`Span`** says where a construct stood in its line, so a cut never goes through one.
- **`parse_inline`** reads one line into its pieces, optionally collecting their spans; **`parse_markdown`**
  reads a whole source into a document.
- **`render_inline`** and **`plain_inline`** write a line's pieces as markup and as plain text.
- **`Converted`** holds a source's document, markup, plain text and whether the markup says anything the
  plain text does not; **`convert_markdown`** makes one.
- **`Fold`** says where a fold falls, by blocks: the opening and the closing stay open and the rest goes into
  the expandable quote. **`fold_blocks`** decides it, or none where too little would be hidden;
  **`folded_markup`** writes the folded message; **`code_markup`** is what a send asking no gate puts up: the
  fold where the message is long enough, the markup where it carries formatting, and none, meaning the plain
  text, otherwise.
- **`sent_length`** is how long a piece of markdown is to Telegram: the longer of its markup's visible text
  and its plain text.
- **`markdown_parts`** divides a long text into consecutive messages at the pauses a reader makes, each under
  the limit the caller gives, a code block cut in two being closed and opened again with its language.
- **`Pair`**, **`Span`** and **`sent_length`** are the crate's own.

### `markdown_structure`

- **`MarkdownHeading`**, **`markdown_headings`**, **`plain_heading`**, **`heading_slug`**,
  **`markdown_sections`**, **`halve_text`**, **`MarkdownStructure`** and **`markdown_structure`** are the
  package's `markdown-structure`; **`Halved`** is what `halve_text` answers, the two halves and the joint.

### `mcp`

The crate's own, compiled only with the `mcp` feature: the MCP server the binary starts (「The MCP server
`bmo-text mcp`」). **`serve_stdio`** answers MCP over the process's standard input and output until the other
side closes them; **`TextTools`** is the server, whose tools are no API of their own beyond what that section
states; **`VERSION`** is the version the server names itself at (「Version numbers」).

### `translation_parts`

- **`TRANSLATION_PART_TOKENS`**, **`translation_parts`** and **`shown_part`** are the package's
  `translation-parts`.

### `web_address`

- **`address_as_written`** takes a candidate address and gives it as it ends: from the first space or Korean
  letter on, the punctuation at its end and each closing bracket it did not open are the sentence's.
- **`is_web_address`** answers whether a text is an http or https address with a host.
- **`MarkdownLink`** and **`markdown_link_at`** read a markdown link starting at an index: its label, its
  address and the index past its closing parenthesis, brackets in the label and one level of parentheses in
  the address balanced.
- **`bare_address_at`** reads the bare address starting at an index, cut where the sentence takes over.
- **`glued_after_address`** answers whether the sentence runs on straight after an address, a Korean letter
  following it or the brackets and punctuation it left.
- **`FoundAddress`**, **`web_addresses`** and **`web_addresses_in`** give every address a text names, in order,
  with where it stands and the words of the markdown link that named it; a markdown link gives the address in
  its parentheses alone.
- The module reads a run of characters, so an index it takes or gives counts characters of that run, where
  the package's counts UTF-16 units of its string; the two name the same places.

### `wording`

The terms a glossary bans, read out of its document and found in a text. The document is markdown with three
level-two sections: banned terms (`<language> · <term> → <what to write>`, optionally ` — <when>`), uses in
the word's own sense (`<language> · <term> · <phrase> — <why>`) and allowed uses (`<language> · <term> ·
<path> · <excerpt> — <why>`).

- **`WORDING_DOCUMENT`** is the document's conventional name, `WORDING.md`; **`KNOWN_LANGUAGES`** are the
  codes a line can be told in.
- **`BannedTerm`**, **`AllowedUse`**, **`Glossary`** and **`WordingHit`** are an entry, an allowed use, the
  document's two lists, and one line using a banned term.
- **`line_language`** tells a line's language by its script: Korean where it holds Hangul, Japanese where it
  holds kana alone, English otherwise.
- **`parse_glossary`** reads the banned terms and **`parse_wording`** both lists, refusing a line naming a
  language no line can be told in and a phrase of a word's own sense that names no banned term or does not
  hold it.
- **`read_glossary`** reads the banned terms of the document in a folder the caller names, and none where it
  cannot be read.
- **`find_banned`** finds every use of a banned term in a text, a line at a time and on lines of the term's
  own language, a Korean term another syllable runs into from the left belonging to that longer word, and a
  term inside a phrase of its own sense passing.
- **`swaps_in`** finds the same uses outside the spans a text quotes in (`QUOTED_SPAN`): what a text being
  written is held to, each hit carrying its line as written.
- **`text_swaps`** is the crate's own: `swaps_in` against the banned terms of the document in a folder the
  caller names, with an English sentence for a document that cannot be read or does not parse.
- **`QUOTED_SPAN`** is the span a text quotes in: backticks or a pair of quotation marks on one line.
  **`banned_terms_in`** gives the banned terms a text uses outside such spans, each once and in the glossary's
  order; **`banned_terms_line`** writes them beside what to write instead.
- **`add_banned_terms`** writes new banned lines after the last entry of the document's list, passing over a
  term already banned in its language and refusing a line that would not read back as written.
- **`check_wording`** holds files, each read through a closure the caller gives, to the glossary: the uses no
  allowed use covers, each a **`FileHit`** of the path and the hit, and the allowed uses that no longer cover
  anything, in a **`WordingReport`**. A read that fails ends the check with the reader's error.

## The TypeScript package `bmo-text`

Each module is imported by its name, as `bmo-text/<module>`. Every module answers through the crate built to
WebAssembly, which the package carries (`docs/design.md`, 「One implementation」).

- **Nothing is awaited.** The build is compiled and instantiated synchronously when the first module is
  imported, so every function may be called as soon as its import has run, in Bun, in Node and in a browser
  bundle (a Nuxt client among them) alike. A bundler needs no plugin for WebAssembly: the bytes are a
  JavaScript module's.
- **A text crosses as UTF-8.** An unpaired surrogate in a string a function is given is read as U+FFFD.
- **A fault in the build throws.** A function whose input the crate cannot read in the package's shape throws
  a `RuntimeError` or an `Error` from the binding rather than answering.

### `answer-cap`

- **`maxAnswerChars`** (30,000) and **`maxValueChars`** (2,000) are the ceilings, in characters, of a verb's
  answer and of one value in it.
- **`capItems`** cuts a list to the whole items whose rendering fits under the ceiling, never inside one, and
  gives the items kept, how many were left and one English line saying so and how to ask for the rest, in the
  caller's unit and words; **`Capped`** is that answer.
- **`capValue`** cuts one long value at the ceiling, counting code points and never tearing a surrogate pair,
  and says how many characters were cut.

### `answer-language`

- **`holdsKorean`** answers whether a text holds any Hangul syllable at all.
- **`TextLanguage`** and **`textLanguage`** tell whether a text is written in Korean or English by its letters
  outside code: a text with few Hangul syllables against its Latin letters is English, every other Korean.

### `dashes`

- **`withoutDashes`** and **`readDashes`** are the crate's `without_dashes` and `read_dashes`, the wrapped
  reading asked for by an option; **`DashReading`** is the latter's `text` and `found`.

### `english`

- **`EnglishTest`** is the crate's `EnglishTest`: constructed from the set-aside pattern, a regular
  expression source as the part writes it, and the Latin letters each Hangul syllable is weighed against,
  both defaulting to the strictest test; **`passes`** answers for a text. The pattern is compiled by the
  crate's engine, which takes lookaround and backreferences and reads `\w`, `\d` and `\b` over all of
  Unicode where JavaScript's reads them over ASCII. It throws on a pattern that does not compile.
- **`holdsNonEnglish`** and **`nonEnglishLetters`** are the crate's `holds_non_english` and
  `non_english_letters`. The stretches are optional and name none by default; **`CodeStretches`** holds
  them as `comments` and `literals`, each a list of **`Stretch`**, a `start` and an `end`.
  **`NonEnglishLetter`** is one letter found, its `letter` a string of one code point and its `place` one of
  **`LetterPlace`**'s `"comment"`, `"literal"` and `"code"`.

### `finding-markdown`

A search finding's structure lines (`VERDICT:`, `SUMMARY:`, `NAME:`, `ADDRESS:`, `MENU:`, `IMAGE:` and the
rest) turned into the markdown they stand for.

- **`PART_KEYS`** are the parts a label opens: the verdict, the summary, the introduction, the body, the
  conclusion, what was not confirmed, and the fold of more.
- **`FindingWords`** are the words a reader sees, the consumer's to choose: what each part is called by its
  key, and the labels of an address, a map, a street view, a release, a newer model, a product page, a stock
  state, the usual price (holding `{usual}`), a deal, a review, the menu and its three columns, and the word a
  search writes for a review with no rating. **`englishFindingWords`** is the default: each part called by its
  key and every label in English.
- **`findingMarkdown`** draws a finding with the given words: a picture or a gallery of pictures written
  together, a menu folded as a table, a map line drawing the place's map, a review as a quotation, footnote
  marks as in-page links.
- **`footnoteMarks`** turns a finding's footnote marks into links; **`pictureCaption`** gives a picture's shown
  caption, its shot and source after it.
- **`FINDING_BLOCK`** and **`FINDING_MAP`** are the shapes of the fold, gallery and map blocks the drawing leaves
  for a renderer.
- **`findingExcerpt`** gives the words a set of findings opens with, as one line of plain prose under a limit,
  cut at a space with an ellipsis: what a link's preview card describes a document by.

### `format`

- **`escapeHtml`**, **`escapeAttribute`**, **`isLinkable`**, **`visibleText`**, **`expandableMarkup`**,
  **`expandable`**, **`longTextLength`**, **`MarkupFault`**, **`markupFault`** and **`faultReason`** are the
  crate's `format`, the fault a tagged union by its kind.

### `html-reading`

- **`readHtml`** reads an HTML page for what it says: its title, its words as markdown-shaped lines in order
  (headings with their level, a list item a dash, each block a line, scripts and styles left out), and every
  picture written inside it as data taken out as a file named from the given stem, an outside picture kept by
  its address. **`HtmlReading`** and **`HtmlPicture`** are that reading and one picture's name, type and bytes.
  It reads any page without throwing: an element that cannot hold content, such as a self-closing one in SVG,
  hides nothing after it, and a picture whose percent escapes are not well formed keeps its payload's bytes as
  they stand.

### `markdown-structure`

- **`MarkdownHeading`** and **`markdownHeadings`** give a text's ATX headings outside fenced code, each with its
  depth, words, mark and line.
- **`plainHeading`** gives a heading's words without their markup; **`headingSlug`** makes its mark from
  letters and digits in any script, a repeated mark numbered.
- **`markdownSections`** cuts a text at each heading, the lead before the first kept, the sections joining back
  into the text; **`halveText`** cuts a text in two at the paragraph boundary nearest its middle, or the
  line boundary where it has no paragraph boundary, and says what joins the halves; none for one line.
- **`MarkdownStructure`** and **`markdownStructure`** give a text's `headings` and its `sections` in one
  reading, as `markdownHeadings` and `markdownSections` give them.

### `markdown`

- **`Inline`**, **`Line`**, **`Block`**, **`MarkdownDocument`**, **`Converted`** and **`Fold`** are the crate's
  reading; **`parseInline`**, **`parseMarkdown`**, **`renderInline`**, **`plainInline`**, **`convertMarkdown`**,
  **`foldBlocks`**, **`foldedMarkup`**, **`codeMarkup`** and **`markdownParts`** its functions, a division
  taking its limit from the caller.

### `translation-parts`

- **`translationParts`** cuts a text into the parts a translator is given: its sections, consecutive ones
  packed together while they fit a token budget (**`TRANSLATION_PART_TOKENS`**, 750, by default), a section
  too large cut at its paragraphs, the parts joining back into the text. **`shownPart`** gives what a part
  shows once read: its translation where it has one, the original otherwise.

### `web-address`

- **`addressAsWritten`**, **`isWebAddress`**, **`MarkdownLink`**, **`markdownLinkAt`**, **`bareAddressAt`**,
  **`gluedAfterAddress`**, **`FoundAddress`** and **`webAddresses`** are the crate's `web_address`, indices
  counted in UTF-16 units.

### `wording`

- **`wordingDocument`**, **`BannedTerm`**, **`AllowedUse`**, **`Glossary`**, **`WordingHit`**, **`lineLanguage`**,
  **`parseGlossary`**, **`addBannedTerms`**, **`findBanned`**, **`swapsIn`**, **`bannedTermsIn`** and
  **`bannedTermsLine`** are the crate's `wording`. **`parseGlossary`** gives both lists; **`readGlossary`** reads
  the document in the folder the caller names and throws where it cannot be read; **`findBanned`** and
  **`swapsIn`** take the path the text was read from, which each hit carries; **`knownLanguages`**, the crate's
  `KNOWN_LANGUAGES`, are the codes a line can be told in.
- **`checkWording`** and **`WordingReport`** are the crate's `check_wording` and `WordingReport`, a hit
  carrying its path itself; an error the reading function throws is thrown on.
- A term the package gives back, alone or in a hit, is the caller's own object from the list it passed.
- The crate's `QUOTED_SPAN`, its `FileHit`, its `text_swaps` and its `parse_glossary` of the banned terms
  alone are the crate's.

## The MCP server `bmo-text mcp`

The crate's binary `bmo-text`, run as `bmo-text mcp`, answers MCP over its standard input and output for the
harness that started it (`docs/design.md`, 「The MCP server」). It is installed with `cargo install --path .`
from the repository's root. It serves tools alone, seven of them, each a thin call of the crate function
named; an input field marked optional may be left out or `null`. A tool's answer is its structured output, the
same JSON written once more as its text; a tool that cannot answer gives an error result of one English
sentence.

- **`text_dashes`** calls `dashes::read_dashes`. In: `text`, and optional `replace` and `wrapped`, both false
  by default. Out: `text`, the text with its dashes replaced where `replace` is true and as given otherwise,
  and `found`, how many dashes between words it holds.
- **`text_language`** calls `answer_language::text_language`. In: `text`. Out: `language`, `ko` or `en`.
- **`text_markdown_structure`** calls `markdown_structure::markdown_structure`. In: `markdown`. Out:
  `headings`, each its `depth`, `text`, `slug` and `line`, and `sections`, the text cut at each heading.
- **`text_html_reading`** calls `html_reading::read_html_page`, the pictures named after `picture`. In: `html`,
  and optional `from`, the index of the first line asked for, nought by default. Out: `title` or `null`,
  `lines`, and `more`, whether lines are left after them; the next page is asked from `from` and the number
  of lines given.
- **`text_translation_parts`** calls `translation_parts::translation_parts`. In: `text`, and optional
  `limit`, the token budget of a part, `TRANSLATION_PART_TOKENS` by default. Out: `parts`.
- **`text_web_addresses`** calls `web_address::web_addresses_in`. In: `text`. Out: `addresses`, each address
  as written, in order.
- **`text_wording`** calls `wording::text_swaps`. In: `text`, and optional `folder`, the folder holding the
  glossary document, the server's working directory by default. Out: `swaps`, each its `line`, the line's
  `text`, the `term`, what to write `instead` and `when` the entry applies, or `null`. A glossary that cannot
  be read or does not parse is the tool's error.

## The harness plugin `bmo-text`

`plugin/` is the plugin a harness installs: its manifest names it `bmo-text` at a placeholder version,
its MCP configuration starts `bmo-text mcp` from the harness's `PATH`, and its skill `bmo-text` tells the
harness when to call each tool.

## Data files

- **`cases/<module>.json`** holds, for each module, the cases both sides are held to (`docs/design.md`,
  「The twins」): an `about` line and, per public function, cases each with a `name`, the inputs and the
  answer expected. A piece of text too long to write out stands as a text and how many times it repeats,
  where the file's `about` says so. **`cases/answer-cap.json`** is one such: lists to cut, with their render,
  unit, ceiling and what is kept, left and said, and values to cut with what they become; its entries of a
  repeated Hangul syllable tell a count of characters from a count of bytes.
- **`korean-ledger.json`** at the root counts, per code file, the lines holding Korean the code inherently
  reads; the repository test and the testbed's `korean` check hold every file to it.

## Version numbers

The version is the set's (bmo-chat's `docs/conventions.md`, 「Versions」): a release is the git tag
`v<version>` on a commit, and a build reads its version off the tags when it is made, which the testbed hands
it as `BMO_VERSION`. The MCP server names that version, or the crate's manifest version where a build was made
without it. The crate's, the package's and the plugin's manifest versions are placeholders that never move, and
nothing shows them as the running version. A number the repository carried under its earlier semver numbering
reads as the revision beside it and names the same API:

| Earlier number | Revision |
| --- | --- |
| `0.1.0` | `0.1.0` |
| `1.0.0` | `0.1.0-rev.1` |
| `2.0.0` | `0.1.0-rev.2` |
| `2.1.0` | `0.1.0-rev.3` |
| `2.2.0` | `0.1.0-rev.4` |
| `3.0.0` | `0.1.0-rev.5` |
