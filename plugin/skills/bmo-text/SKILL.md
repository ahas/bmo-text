---
name: "bmo-text"
description: "When and how to call the text tools of the bmo-text MCP server: before sending or filing text, to find or replace dashes between words, tell whether a text is Korean or English, read a markdown text's headings and sections, read an HTML page into lines, cut a text into translation parts, list the web addresses a text names, or hold a text to a project's glossary of banned terms."
---

# Text tools

The `bmo-text` MCP server answers seven tools, each a pure reading of the text it is given. None of them
writes a file, keeps anything between calls or reaches the network. Install the server with
`cargo install --path .` from the bmo-text repository, so `bmo-text` is on the `PATH` the harness starts it
from.

## Before text leaves the session

- **Dashes.** A dash between words (an em dash, a bar, a spaced en dash) reads as machine writing. Call
  `text_dashes` on a chat answer or a document before it goes out: `found` says how many there are, and
  `replace: true` gives the text back with each one written as a person would type the sentence (a colon,
  commas around an aside, a range between digits). Pass `wrapped: true` for a document wrapped at a fixed
  width, so a paragraph is read as one piece across its line breaks; leave it out for a chat answer, whose
  every line stands alone. Dashes in code, quotations, addresses and tables are the text's own and are left.
- **Banned terms.** Where the project keeps a glossary (`WORDING.md`), call `text_wording` on what you wrote.
  Each swap names the line, the term and what to write instead; a term named inside backticks or quotation
  marks passes. The glossary is read from the folder you name in `folder`, or from the folder the server was
  started in. An error result means the glossary could not be read or does not parse: say so rather than
  taking the text as clean.
- **Language.** `text_language` answers `ko` or `en` by the letters outside code, for holding an answer to
  the language it must be written in.

## Reading what you were given

- **Markdown structure.** `text_markdown_structure` gives the headings outside fenced code (depth, words,
  slug, line) and the sections the text is cut into at each heading, which join back into the text: use it
  for a table of contents, a link to a heading, or to work on a long document a section at a time.
- **An HTML page.** `text_html_reading` reads a page into its title and lines, a block to a line, headings and
  list items in markdown, scripts and styles left out. A long page comes a part at a time: while `more` is
  true, call again with `from` set to the last `from` plus the number of lines you received.
- **Web addresses.** `text_web_addresses` lists the http and https addresses a text names, bare or in
  markdown links, with the sentence's punctuation left off their ends.
- **Translation parts.** `text_translation_parts` cuts a text into the parts to translate one at a time: its
  sections packed together under `limit` estimated tokens (750 by default), a section too large cut at its
  paragraphs. Translate each part on its own and join the results in order.
