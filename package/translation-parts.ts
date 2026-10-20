// How a text is read for the translator: the parts it is cut into and joined back from. Which language it is
// written in is `answer-language`'s reading. The readings are the crate's, through the package's WebAssembly
// build.

import { binding } from "./binding";

// How much of a text one translation part carries, in estimated tokens. The translation of a part has
// to fit the translator's answer of 2,048 tokens, and a translation into Korean costs more tokens than
// the English it renders. Parts this small are what let the first one show in seconds.
export const TRANSLATION_PART_TOKENS = binding.translationPartTokens() as 750;

// A text as the parts the translator is given: its sections, the consecutive ones packed together
// while they fit, a section too large on its own cut at its paragraphs. Joined back, the parts are
// the text.
export function translationParts(text: string, budget: number = TRANSLATION_PART_TOKENS): string[] {
  return binding.translationParts(text, budget);
}

// One part as it stands once read: its translation where there is one, the original where there is
// none. The translator trims what it hands back, so a part that ended the original's line ends the
// translation's too and the parts after it stay blocks of their own.
export function shownPart(source: string, text: string | null): string {
  return binding.shownPart(source, text);
}
