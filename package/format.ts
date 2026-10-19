// Telegram's HTML parse mode, as a chat service uses it: the escapes, the one quote a fold is made of, and the
// structure check every markup is read by before it is sent. The readings are the crate's, through the
// package's WebAssembly build.

import { binding } from "./binding";

// In HTML parse mode only these three characters carry meaning outside a tag.
export function escapeHtml(text: string): string {
  return binding.escapeHtml(text);
}

// An attribute value is delimited by double quotes as well, so it needs one more.
export function escapeAttribute(value: string): string {
  return binding.escapeAttribute(value);
}

// Whether Telegram takes an address in an href: http or https and no space; anything else makes the whole
// message fail.
export function isLinkable(url: string): boolean {
  return binding.isLinkable(url);
}

// What a reader is left with once the tags come out and the three escapes go back: the words
// of a piece of markup and nothing else.
export function visibleText(markup: string): string {
  return binding.visibleText(markup);
}

// markup is HTML already built from escaped text; the quote only wraps it.
export function expandableMarkup(markup: string): string {
  return binding.expandableMarkup(markup);
}

export function expandable(text: string): string {
  return binding.expandable(text);
}

// Past this many characters a message is long enough that hiding the bulk of it helps.
export const longTextLength = binding.longTextLength() as 600;

// What the check found, said in pieces so the sentence a room reads is written where the other
// sentences are, and a test can name the fault rather than match its prose.
export type MarkupFault =
  | { kind: "unescaped"; character: string }
  | { kind: "unsupported"; tag: string }
  | { kind: "unclosed"; tag: string }
  | { kind: "unopened"; tag: string; open: string | null }
  | { kind: "nested"; tag: string; inside: string };

// A fault the code found, said in English: which tag, and where it was. A reader wanting it in another
// language reads the fault's kind and its data instead.
export function faultReason(fault: MarkupFault): string {
  return binding.faultReason(JSON.stringify(fault));
}

// The first fault in a piece of markup, or null when it has none. The tags are read for what they
// are and how they pair and nest, and the characters between them for whether the escaping held.
export function markupFault(markup: string): MarkupFault | null {
  return binding.markupFault(markup);
}
