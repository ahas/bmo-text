// An agent writes markdown and Telegram reads HTML. This module is the one place the first becomes the
// second: the markdown is read once, and the markup sent, the plain text held as its fallback and the blocks
// the fold and the division work on all come out of that one reading, so none of them can say something the
// others do not. Telegram shows a message line for line, so the reading is line-based rather than
// CommonMark's. The reading is the crate's, through the package's WebAssembly build.

import { binding } from "./binding";
import { telegramTextLimit } from "./telegram";

export type Inline =
  | { kind: "text"; text: string }
  | { kind: "code"; text: string }
  | { kind: "b" | "i" | "s" | "spoiler"; children: Inline[] }
  | { kind: "link"; url: string; children: Inline[] };

export type Line = Inline[];

export type Block =
  | { kind: "lines"; lines: Line[] }
  | { kind: "pre"; language: string | null; text: string }
  | { kind: "quote"; blocks: Block[]; gaps: string[] };

// A document is its blocks and, before each, the line breaks that stood between it and the one
// before, so a message comes out spaced the way it was written. The first gap is always empty.
export interface MarkdownDocument {
  blocks: Block[];
  gaps: string[];
}

export interface Converted {
  source: string;
  document: MarkdownDocument;
  markup: string;
  plain: string;
  // Whether the markup says anything the plain text does not: a tag, or an escape the plain text
  // would not need. A message without it goes out as its plain text.
  formatted: boolean;
}

// Where a construct stood in the line it was read from, in UTF-16 units: the code spans and links, which are
// literal to the end, and the formatting pairs.
interface InlineSpan {
  start: number;
  end: number;
  literal: boolean;
}

// One line of markdown as inline nodes. Nothing here crosses a line break: the caller hands over
// one line at a time. spans, when given, collects where each construct of the line stood.
export function parseInline(text: string, spans?: InlineSpan[]): Inline[] {
  const read: { line: Inline[]; spans: InlineSpan[] } = binding.parseInline(text, spans !== undefined);

  spans?.push(...read.spans);

  return read.line;
}

export function parseMarkdown(source: string): MarkdownDocument {
  return binding.parseMarkdown(source);
}

// A line's markup. A code piece closes every tag around it and reopens them after, which splits a formatting
// around the code it held, and every tag comes out paired and properly nested.
export function renderInline(nodes: Inline[]): string {
  return binding.renderInline(JSON.stringify(nodes));
}

export function plainInline(nodes: Inline[]): string {
  return binding.plainInline(JSON.stringify(nodes));
}

export function convertMarkdown(source: string): Converted {
  return binding.convertMarkdown(source);
}

// Where a fold falls, decided on blocks: the opening and the closing stay in the open and the
// hidden blocks go behind the quote. A code block or a quote is one block whatever blank lines it
// holds, so the fold never cuts through one, and neither is ever the conclusion.
export interface Fold {
  opening: Block | null;
  // The line breaks between the opening and the quote.
  openingGap: string;
  hidden: Block[];
  // The line breaks before each hidden block, the first of them empty.
  hiddenGaps: string[];
  closing: Block | null;
  closingGap: string;
}

// The fold of a document, or null where too little would be hidden.
export function foldBlocks(document: MarkdownDocument): Fold | null {
  return binding.foldBlocks(JSON.stringify(document));
}

// The message folded, or null where there is too little to hide for a fold to help.
export function foldedMarkup(converted: Converted): string | null {
  return binding.foldedMarkup(JSON.stringify(converted)) ?? null;
}

// What a send path that asks no gate puts up: the fold the code makes when the message is long
// enough for one, the converted markup when it carries formatting, and null, the plain text,
// otherwise. Markup the structure check refuses is never it.
export function codeMarkup(converted: Converted): string | null {
  return binding.codeMarkup(JSON.stringify(converted)) ?? null;
}

// A longer text divided into consecutive messages the way a reader pauses: the last blank line, else the last
// line break, else the last space inside the window. Each part is a whole piece of markdown: a code block cut
// in two is closed at the end of one part and opened again, with its language, at the start of the next. The
// parts put back together are the text, save for the fence lines a divided code block gains.
export function markdownParts(text: string, limit: number = telegramTextLimit): string[] {
  return binding.markdownParts(text, limit);
}
