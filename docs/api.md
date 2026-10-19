# Public API

Everything a consumer may use, project by project and module by module. An item not named here is internal
and may change in any version. Lengths are UTF-16 units and spaces are JavaScript's white space on both sides
unless an entry says otherwise; where a module stands on both sides, the two give one answer for one text.

## The Rust crate `bmo-markup`

### `dashes`

- **`without_dashes`** takes a text and whether it is wrapped, and gives the text with every dash left between
  words replaced the way a person types the sentence: a name and its explanation joined by a colon, an aside
  between two dashes set off by commas (or brackets where it has commas of its own), any further dash of the
  sentence a colon or a comma, an unspaced dash between digits a range. What is not prose keeps its dashes:
  code spans and blocks, quotations, quoted lines, table rows, addresses, a dash opening or ending a line. A
  wrapped text reads a paragraph or a list item as one piece across its line breaks; an unwrapped one reads
  each line alone, as a chat answer is written.

### `english`

- **`EnglishTest`** is the test of whether a text is English, as a part's policy declares it.
  - **`new`** takes the set-aside pattern, a regular expression of the words read past wherever they stand,
    and the number of Latin letters each Hangul syllable left is weighed against. It panics on a pattern that
    does not compile.
  - **`passes`** answers whether a text is English: with a weight of zero, no Hangul syllable may be left
    once the set-aside words are read past; otherwise the text holds some Latin letters and fewer syllables
    than one to every so many of them.
  - **`Default`** reads nothing past and weighs at zero, so any Hangul syllable fails it.

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

### `js`

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
  a limit, a code block cut in two being closed and opened again with its language; **`telegram_parts`** does
  it at Telegram's text limit.

### `telegram`

- **`TEXT_LIMIT`** (4096) and **`CAPTION_LIMIT`** (1024) are what one message's text and one file's caption
  hold.
- **`plain_parts`** divides plain words into messages under a limit the way a Telegram client divides a long
  paste: each part ends after the last blank line, line break or space that fits.
- **`transcript_parts`** folds the reading a voice was spoken from under the voice: its first part the
  caption, the rest replies, every part an expandable quote of its escaped words, cut only between whole
  characters a reader sees as one.

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
  with where it stands; a markdown link gives the address in its parentheses alone.

### `wording`

The terms a glossary bans, read out of its document and found in a text. The document is markdown with three
level-two sections: banned terms (`<language> · <term> → <what to write>`, optionally ` — <when>`), uses in
the word's own sense (`<language> · <term> · <phrase> — <why>`) and allowed uses (`<language> · <term> ·
<path> · <excerpt> — <why>`).

- **`WORDING_DOCUMENT`** is the document's conventional name, `WORDING.md`.
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
- **`QUOTED_SPAN`** is the span a text quotes in: backticks or a pair of quotation marks on one line.
  **`banned_terms_in`** gives the banned terms a text uses outside such spans, each once and in the glossary's
  order; **`banned_terms_line`** writes them beside what to write instead.
- **`add_banned_terms`** writes new banned lines after the last entry of the document's list, passing over a
  term already banned in its language and refusing a line that would not read back as written.

## The TypeScript package `bmo-markup`

Each module is imported by its name, as `bmo-markup/<module>`.

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

### `dashes`

- **`withoutDashes`** is the crate's `without_dashes`, the wrapped reading asked for by an option.

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

### `markdown-structure`

- **`MarkdownHeading`** and **`markdownHeadings`** give a text's ATX headings outside fenced code, each with its
  depth, words, mark and line.
- **`plainHeading`** gives a heading's words without their markup; **`headingSlug`** makes its mark from
  letters and digits in any script, a repeated mark numbered.
- **`markdownSections`** cuts a text at each heading, the lead before the first kept, the sections joining back
  into the text; **`halveText`** cuts a text in two at the paragraph boundary nearest its middle, or the
  line boundary where it has no paragraph boundary, and says what joins the halves; none for one line.

### `markdown`

- **`Inline`**, **`Line`**, **`Block`**, **`MarkdownDocument`**, **`Converted`** and **`Fold`** are the crate's
  reading; **`parseInline`**, **`parseMarkdown`**, **`renderInline`**, **`plainInline`**, **`convertMarkdown`**,
  **`foldBlocks`**, **`foldedMarkup`**, **`codeMarkup`** and **`markdownParts`** its functions, the limit of a
  division defaulting to Telegram's text limit.

### `telegram-parts`

- **`telegramTextLimit`**, **`telegramCaptionLimit`** and **`maxSendFileBytes`** are what one message, one
  caption and one file sent through the Bot API hold; **`botIdFromToken`** reads a bot's user id out of its
  token.

### `translation-parts`

- **`TextLanguage`** and **`textLanguage`** tell whether a text is written in Korean or English by its letters
  outside code.
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
  **`parseGlossary`**, **`addBannedTerms`**, **`findBanned`**, **`bannedTermsIn`** and **`bannedTermsLine`** are
  the crate's `wording`. **`parseGlossary`** gives both lists; **`readGlossary`** reads the document in the
  folder the caller names and throws where it cannot be read; **`findBanned`** takes the path the text was
  read from, which each hit carries; **`knownLanguages`** are the codes a line can be told in.
- **`checkWording`** holds files, each read through a function the caller gives, to the glossary: the uses no
  allowed use covers, and the allowed uses that no longer cover anything, in a **`WordingReport`**.

## Data files

- **`ts/tests/answer-cap-cases.json`** holds the cases the answer ceiling is held to: lists to cut, with their
  render, unit, ceiling and what is kept, left and said, and values to cut with what they become. A piece of
  text is a string or a text repeated so many times. A Rust twin of the ceiling a consumer keeps is held to the
  same file; its entries of a repeated Hangul syllable tell a count of characters from a count of bytes.
- **`korean-ledger.json`** at the root counts, per code file, the lines holding Korean the code inherently
  reads; the repository test and the testbed's `korean` check hold every file to it.
