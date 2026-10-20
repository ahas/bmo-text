// The structure of a markdown text, read off its source: its headings, and where it may be cut. Only ATX
// headings count, a line of one to six `#`; a line inside a fenced code block is code, whatever it starts with,
// and no text is cut inside one. The readings are the crate's, through the package's WebAssembly build.

import { binding } from "./binding";

export interface MarkdownHeading {
  // 1 for `#`, 6 for `######`.
  depth: number;
  // What the heading says, with its inline markup taken off.
  text: string;
  // A mark no other heading of the same text carries, made from its words: the first of a text
  // takes the words alone, the ones after it a number as well.
  slug: string;
  // The source line the heading stands on, counted from 1.
  line: number;
}

// A heading's words without the markup around them: a link or an image stands as its words, and
// emphasis, strikes and code marks fall away.
export function plainHeading(text: string): string {
  return binding.plainHeading(text);
}

// The words of a heading as a mark: lower-case, letters and digits of any script kept, runs of
// space made one hyphen, everything else dropped.
export function headingSlug(text: string): string {
  return binding.headingSlug(text);
}

// Every heading of a text, in the order it stands. A heading with no words is no entry.
export function markdownHeadings(source: string): MarkdownHeading[] {
  return binding.markdownHeadings(source);
}

// A text in two, at the paragraph boundary nearest its middle, or the line boundary nearest it
// where there is no paragraph boundary, with what joins the halves back; null where it is one line.
export function halveText(text: string): { halves: [string, string]; joint: string } | null {
  return binding.halveText(text);
}

// A text as its sections: the lines before the first heading, then each heading with the lines
// under it up to the next heading of any depth. Joined back, the sections are the text.
export function markdownSections(source: string): string[] {
  return binding.markdownSections(source);
}

// A markdown text's structure in one reading: its headings and its sections.
export interface MarkdownStructure {
  headings: MarkdownHeading[];
  sections: string[];
}

// A text's headings and its sections, as `markdownHeadings` and `markdownSections` give them.
export function markdownStructure(source: string): MarkdownStructure {
  return binding.markdownStructure(source);
}
