// A dash left between words, replaced the way a person types the same sentence: a name and its explanation get
// a colon, an aside between two dashes of one sentence gets commas (or brackets, where it has commas of its
// own), every further dash in the sentence a comma, and a dash between two digits is a range. What is not prose
// keeps its dashes. A wrapped text is read with each paragraph or list item as one piece; an unwrapped one, as
// a chat answer is written, a line at a time. The reading is the crate's, through the package's WebAssembly
// build.

import { binding } from "./binding";

export function withoutDashes(text: string, options: { wrapped?: boolean } = {}): string {
  return binding.withoutDashes(text, options.wrapped ?? false);
}
