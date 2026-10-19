// A text read for whether it is written in English, by the policy of the part reading it: the words it reads
// past and how many Latin letters a Hangul syllable is weighed against. Both are the part's own, passed in
// from its own policy. The test is the crate's, through the package's WebAssembly build.

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
