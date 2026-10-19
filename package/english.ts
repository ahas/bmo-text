// A text read for whether it is written in English, by the policy of the part reading it: the words it reads
// past and how many Latin letters a Hangul syllable is weighed against. Both are the part's own, passed in
// from its own policy.

const hangul = /[가-힣]/gu;
const latin = /[A-Za-z]/g;

// The test of whether a text is written in English, as one part's policy declares it: the set-aside words
// read past, and the Hangul syllables left read against the Latin letters left. The strictest test reads
// nothing past, and any Hangul syllable fails it.
export class EnglishTest {
  readonly #setAside: RegExp;
  readonly #latinPerHangul: number;

  // A test reading past what `setAside` matches, a regular expression as the part writes it, and weighing
  // each Hangul syllable left against `latinPerHangul` Latin letters; at nought, any Hangul left fails it.
  // A pattern that does not compile throws.
  constructor(setAside = "", latinPerHangul = 0) {
    this.#setAside = new RegExp(setAside, "gu");
    this.#latinPerHangul = latinPerHangul;
  }

  // Whether a text is English: with no Latin letters per Hangul syllable, none left at all; otherwise some
  // Latin letters, and fewer Hangul syllables than one to every so many of them.
  passes(text: string): boolean {
    const prose = text.replace(this.#setAside, "");
    const syllables = prose.match(hangul)?.length ?? 0;

    if (this.#latinPerHangul === 0) {
      return syllables === 0;
    }

    const letters = prose.match(latin)?.length ?? 0;

    return letters > 0 && syllables * this.#latinPerHangul < letters;
  }
}
