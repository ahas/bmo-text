// The web addresses a text names, found the way a chat app finds them. A markdown link names the address its
// parentheses hold and nothing after them. A bare http or https address runs to the next space or Korean
// letter, and what the sentence put after it (a full stop, a comma, a closing bracket that opened before the
// address) is the sentence's, not the address's. Every index is in UTF-16 units. The readings are the
// crate's, through the package's WebAssembly build.

import { binding } from "./binding";

// A candidate address as it ends, with what the sentence put after it taken off: everything from
// the first space or Korean letter on, the punctuation at its end, and each closing bracket it did
// not open.
export function addressAsWritten(candidate: string): string {
  return binding.addressAsWritten(candidate);
}

export function isWebAddress(url: string): boolean {
  return binding.isWebAddress(url);
}

export interface MarkdownLink {
  label: string;
  url: string;
  end: number;
}

// [label](url) starting at `at`, where `at` is the opening bracket. Brackets in the label and one
// level of parentheses in the url are balanced; the url holds no space.
export function markdownLinkAt(text: string, at: number): MarkdownLink | null {
  return binding.markdownLinkAt(text, at);
}

// The bare address that starts exactly at `at`, cut where the sentence takes over, or null where
// none starts there.
export function bareAddressAt(text: string, at: number): string | null {
  return binding.bareAddressAt(text, at) ?? null;
}

// Whether the sentence runs on with no space after the address that ends at `end`: a Korean letter
// straight after it, or after the closing brackets and punctuation it left behind, which a chat app
// that reads addresses by its own rule would take into the address.
export function gluedAfterAddress(text: string, end: number): boolean {
  return binding.gluedAfterAddress(text, end);
}

export interface FoundAddress {
  // Where the address stands in the text, end exclusive: inside a markdown link's parentheses, or
  // the bare address itself.
  start: number;
  end: number;
  url: string;
  // The words of the markdown link that named it, or null for a bare address.
  label: string | null;
}

// Every web address a text names, in the order written. A markdown link, a picture's mark included,
// gives the address in its parentheses and nothing else.
export function webAddresses(text: string): FoundAddress[] {
  return binding.webAddresses(text);
}
