// What language an answer is written in: whether a text holds any Korean, which an answer held to the room's
// language is read for, and which language a text is written in, which a translator is told. The readings are
// the crate's, through the package's WebAssembly build.

import { binding } from "./binding";

/** Whether a piece of text has any Korean in it at all. */
export function holdsKorean(text: string): boolean {
  return binding.holdsKorean(text);
}

export type TextLanguage = "ko" | "en";

// Which language a text is written in, by its letters outside code: a text with few Hangul syllables
// against its Latin letters is English, and every other text is Korean.
export function textLanguage(text: string): TextLanguage {
  return binding.textLanguage(text) as TextLanguage;
}
