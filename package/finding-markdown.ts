// A search finding's structure lines, turned into the markdown they stand for. Nothing here touches a DOM, and
// the marks it leaves (a heading per part, a heading per name) are what every reader of the markdown counts on
// standing in the same places. The drawing is the crate's, through the package's WebAssembly build.

import { binding } from "./binding";

// The parts a label opens.
export const PART_KEYS: readonly string[] = binding.partKeys();

// The words a reader sees in a drawn finding, which are the consumer's to choose and read from its
// own language pack: what each part is called, by its key, and the label of each line an entry
// draws. `usual` holds `{usual}` where the usual price goes, and `noRating` is the word a search
// writes in a review's rating where the source gave none.
export interface FindingWords {
  parts: Readonly<Record<string, string>>;
  address: string;
  map: string;
  streetView: string;
  released: string;
  newer: string;
  productPage: string;
  stock: string;
  usual: string;
  deal: string;
  review: string;
  noRating: string;
  menu: string;
  dish: string;
  price: string;
  notes: string;
}

// The words a finding is drawn with where its consumer passes none: each part called by its key, and
// every line's label in English.
export const englishFindingWords: FindingWords = binding.englishFindingWords();

// A finding's footnote marks, [3] or [3†] for a value read only in a search snippet, turned into the in-page
// links they stand for. A bracket followed by an address is a link of its own.
export function footnoteMarks(line: string): string {
  return binding.footnoteMarks(line);
}

// What a picture's line is captioned with where it is shown, out of the fields after its address: what
// it shows, then its shot and whose picture it is where the line names them.
export function pictureCaption(captioned: string): string {
  return binding.pictureCaption(captioned);
}

// The blocks a finding's structure lines make that markdown has no form for: a fold with its label,
// a gallery of pictures, and the map of a place. A fold and a gallery open on `::: <kind>` and close
// on a line of `:::` alone; a map is its one line, naming the place to find. The finding renderer
// draws them, and every other reader of this markdown takes them for paragraphs.
export const FINDING_BLOCK = /^::: (fold|gallery)(?: ([^\n]*))?\n([\s\S]*?)\n:::(?:\n|$)/;
export const FINDING_MAP = /^::: map ([^\n]+)(?:\n|$)/;

// A finding's structure lines as markdown. A picture is a paragraph of its own and a set's divider
// is a rule, so both are set apart by blank lines. Pictures written together are one gallery, a
// menu is a fold of its own, and a map line draws the place's map above its link.
export function findingMarkdown(text: string, words: FindingWords = englishFindingWords): string {
  return binding.findingMarkdown(text, JSON.stringify(words));
}

// The words a set of findings opens with, as one line of plain prose no longer than `limit`: what a
// place that shows a document by a few words and draws no markup is given, such as the card a chat
// app draws for a link to it. A line too long is cut at the last space before the limit and closed
// with an ellipsis. Empty where the findings say nothing in prose.
export function findingExcerpt(texts: readonly string[], limit: number): string {
  return binding.findingExcerpt([...texts], limit);
}
