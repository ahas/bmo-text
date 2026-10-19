// A text read for whether it is written in English, two ways. By the policy of the part reading it: the
// words it reads past and how many Latin letters a Hangul syllable is weighed against, both the part's own,
// passed in from its own policy. And by script alone, whatever the language: every letter of a script but
// Latin, written as itself or named by an escape, placed in a comment, a literal or the code by the
// stretches the caller's reader of the code language marks. Every index is in UTF-16 units. The readings
// are the crate's, through the package's WebAssembly build.

import { binding } from "./binding";
import type { EnglishTest as CompiledTest } from "./wasm/bmo_text.js";

// The test of whether a text is written in English, as one part's policy declares it: the set-aside words
// read past, and the Hangul syllables left read against the Latin letters left. The strictest test reads
// nothing past, and any Hangul syllable fails it.
export class EnglishTest {
  readonly #test: CompiledTest;

  // A test reading past what `setAside` matches, a regular expression as the part writes it, and weighing
  // each Hangul syllable left against `latinPerHangul` Latin letters; at nought, any Hangul left fails it.
  // A pattern that does not compile throws.
  constructor(setAside = "", latinPerHangul = 0) {
    this.#test = new binding.EnglishTest(setAside, latinPerHangul);
  }

  // Whether a text is English: with no Latin letters per Hangul syllable, none left at all; otherwise some
  // Latin letters, and fewer Hangul syllables than one to every so many of them.
  passes(text: string): boolean {
    return this.#test.passes(text);
  }
}

// Whether a text holds any letter that is not English: a letter of any script but Latin, standing as itself
// or named by a braced `\u{…}`, a four-digit `\u`, an eight-digit `\U` or a pattern's braced `\x{…}`.
export function holdsNonEnglish(text: string): boolean {
  return binding.holdsNonEnglish(text);
}

// A stretch of a text, end exclusive.
export interface Stretch {
  start: number;
  end: number;
}

// The comments and the literals of a code text, as the caller's own reader of its language marks them.
export interface CodeStretches {
  comments: Stretch[];
  literals: Stretch[];
}

// Where a letter stands: in a comment, in a literal, or in neither, the code itself.
export type LetterPlace = "comment" | "literal" | "code";

export interface NonEnglishLetter {
  // Where the letter, or the whole escape naming it, stands, end exclusive.
  start: number;
  end: number;
  // The letter, one code point.
  letter: string;
  // Whether an escape names the letter rather than the letter standing as itself.
  escaped: boolean;
  place: LetterPlace;
}

// Every letter of a text that is not English, in the order written, placed by `stretches`; with none named,
// every letter is in the code.
export function nonEnglishLetters(
  text: string,
  stretches: CodeStretches = { comments: [], literals: [] },
): NonEnglishLetter[] {
  return binding.nonEnglishLetters(text, JSON.stringify(stretches));
}
