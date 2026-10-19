// The web addresses a text names, found the way a chat app finds them. A markdown link names the
// address its parentheses hold and nothing after them. A bare http or https address runs to the
// next space or Korean letter, and what the sentence put after it — a full stop, a comma, a closing
// bracket that opened before the address — is the sentence's, not the address's. A pair of
// brackets inside the address, as a wiki page's name carries, stays in it.

// Korean letters: jamo, compatibility jamo, and the syllables with their extensions. An address
// written straight into a Korean sentence ends where the sentence's own letters begin, as an address
// with a particle glued to it does.
const HANGUL = "ᄀ-ᇿ㄰-㆏ꥠ-꥿가-힯ힰ-퟿";
const BARE = new RegExp(`https?:\\/\\/[^\\s<>"'\`${HANGUL}]+`, "iy");
const SENTENCE_LETTER = new RegExp(`[\\s<>"'\`${HANGUL}]`);
const GLUED = new RegExp(`^[)\\]}.,;:!?]*[${HANGUL}]`);
const TRAILING = /[.,;:!?'"…。、」』》）]+$/;
const BRACKETS = [
  ["(", ")"],
  ["[", "]"],
  ["{", "}"]
] as const;

// A candidate address as it ends, with what the sentence put after it taken off: everything from
// the first space or Korean letter on, the punctuation at its end, and each closing bracket it did
// not open.
export function addressAsWritten(candidate: string): string {
  const letter = SENTENCE_LETTER.exec(candidate);
  let url = (letter === null ? candidate : candidate.slice(0, letter.index)).replace(TRAILING, "");

  for (const [open, close] of BRACKETS) {
    while (url.endsWith(close) && url.split(open).length < url.split(close).length) {
      url = url.slice(0, -1).replace(TRAILING, "");
    }
  }

  return url;
}

export function isWebAddress(url: string): boolean {
  try {
    const parsed = new URL(url);

    return (parsed.protocol === "http:" || parsed.protocol === "https:") && parsed.hostname !== "";
  } catch {
    return false;
  }
}

export interface MarkdownLink {
  label: string;
  url: string;
  end: number;
}

// [label](url) starting at `at`, where `at` is the opening bracket. Brackets in the label and one
// level of parentheses in the url are balanced; the url holds no space.
export function markdownLinkAt(text: string, at: number): MarkdownLink | null {
  let depth = 0;
  let close = -1;

  for (let index = at; index < text.length; index += 1) {
    const character = text[index];

    if (character === "\\") {
      index += 1;
      continue;
    }

    if (character === "[") {
      depth += 1;
    } else if (character === "]") {
      depth -= 1;

      if (depth === 0) {
        close = index;
        break;
      }
    }
  }

  if (close === -1 || text[close + 1] !== "(") {
    return null;
  }

  let parens = 0;

  for (let index = close + 2; index < text.length; index += 1) {
    const character = text[index] ?? "";

    if (/\s/.test(character)) {
      return null;
    }

    if (character === "(") {
      parens += 1;
    } else if (character === ")") {
      if (parens === 0) {
        const url = text.slice(close + 2, index);

        return url === "" ? null : { label: text.slice(at + 1, close), url, end: index + 1 };
      }

      parens -= 1;
    }
  }

  return null;
}

// The bare address that starts exactly at `at`, cut where the sentence takes over, or null where
// none starts there.
export function bareAddressAt(text: string, at: number): string | null {
  BARE.lastIndex = at;
  const match = BARE.exec(text);

  if (match === null) {
    return null;
  }

  const url = addressAsWritten(match[0]);

  return url !== "" && isWebAddress(url) ? url : null;
}

// Whether the sentence runs on with no space after the address that ends at `end` — a Korean
// letter straight after it, or after the closing brackets and punctuation it left behind — which a
// chat app that reads addresses by its own rule would take into the address.
export function gluedAfterAddress(text: string, end: number): boolean {
  return GLUED.test(text.slice(end));
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

// Every web address a text names, in the order written. A markdown link — a picture's mark
// included — gives the address in its parentheses and nothing else.
export function webAddresses(text: string): FoundAddress[] {
  const found: FoundAddress[] = [];
  let index = 0;

  while (index < text.length) {
    const character = text[index];

    if (character === "[") {
      const link = markdownLinkAt(text, index);

      if (link !== null) {
        const start = link.end - 1 - link.url.length;

        if (isWebAddress(link.url)) {
          found.push({ start, end: link.end - 1, url: link.url, label: link.label });
        }

        index = link.end;
        continue;
      }
    }

    if (character === "h" || character === "H") {
      const url = bareAddressAt(text, index);

      if (url !== null) {
        found.push({ start: index, end: index + url.length, url, label: null });
        index += url.length;
        continue;
      }
    }

    index += 1;
  }

  return found;
}
