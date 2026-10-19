// Whether a text holds any Korean, which an answer held to the room's language is read for. The reading is the
// crate's, through the package's WebAssembly build.

import { binding } from "./binding";

/** Whether a piece of text has any Korean in it at all. */
export function holdsKorean(text: string): boolean {
  return binding.holdsKorean(text);
}
