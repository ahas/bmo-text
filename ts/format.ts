// Telegram's HTML parse mode, as a chat service uses it: the escapes, the one quote a fold is made
// of, and the structure check every markup is read by before it is sent. An agent writes markdown
// and the markup is built from it in markdown.ts; every send path keeps the plain text of the same
// reading as its fallback, so a rejected markup costs the shape of a message and never the message.

// In HTML parse mode only these three characters carry meaning outside a tag.
export function escapeHtml(text: string): string {
  return text.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}

// An attribute value is delimited by double quotes as well, so it needs one more.
export function escapeAttribute(value: string): string {
  return escapeHtml(value).replaceAll('"', "&quot;");
}

// Telegram accepts only these schemes in an href; anything else makes the whole message fail.
const linkableUrl = /^https?:\/\/\S+$/i;

export function isLinkable(url: string): boolean {
  return linkableUrl.test(url);
}

// What a reader is left with once the tags come out and the three escapes go back: the words
// of a piece of markup and nothing else. Markup that leaves nothing here is markup around no
// message, which is the one thing a formatted attempt can be that is not worth sending.
export function visibleText(markup: string): string {
  return markup
    .replaceAll(/<[^>]*>/g, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&amp;", "&");
}

// markup is HTML already built from escaped text; the quote only wraps it.
export function expandableMarkup(markup: string): string {
  return `<blockquote expandable>${markup}</blockquote>`;
}

export function expandable(text: string): string {
  return expandableMarkup(escapeHtml(text));
}

// Past this many characters a message is long enough that hiding the bulk of it helps.
export const longTextLength = 600;

// What the markup rulebook says about tags, in the form the code can run. Telegram answers a
// tag it does not know, or one that never closes, by refusing the whole message — the room is
// left with nothing — and the nesting it refuses is refused the same way. None of that is a
// judgement: it is read off the markup by pairing tags, so it is settled here rather than
// asked of a model that can only look at the text and guess.
const tagAt = /<(\/?)([a-zA-Z][\w-]*)((?:\s[^>]*)?)>/y;

// Bold, italic, underline, strikethrough and spoiler. These hold one another freely.
const formattingTags = new Set(["b", "strong", "i", "em", "u", "ins", "s", "strike", "del", "tg-spoiler", "span"]);

// A literal: no formatting goes inside one, and no formatting holds one. A code tag directly
// inside a pre tag is the one exception, because that is how a code block names its language.
const literalTags = new Set(["code", "pre"]);

// The rest of what the rulebook lists as usable. Custom emoji is missing on the rulebook's own
// word: Telegram supports it and this bot does not use it, so it is not a tag the gate may send.
const otherTags = new Set(["a", "blockquote", "tg-time"]);

// A span is a spoiler or it is nothing: the bare tag is not one Telegram knows.
const spoilerClass = /(?:^|\s)class="tg-spoiler"(?:\s|$)/;

// What the check found, said in pieces so the sentence a room reads is written where the other
// sentences are, and a test can name the fault rather than match its prose.
export type MarkupFault =
  | { kind: "unescaped"; character: string }
  | { kind: "unsupported"; tag: string }
  | { kind: "unclosed"; tag: string }
  | { kind: "unopened"; tag: string; open: string | null }
  | { kind: "nested"; tag: string; inside: string };

// Whether a tag may open where the tags already open put it.
function nestingFault(tag: string, stack: string[]): MarkupFault | null {
  const literal = stack.find((open) => literalTags.has(open));

  if (literal !== undefined) {
    const languageNaming = literal === "pre" && tag === "code" && stack[stack.length - 1] === "pre";

    return languageNaming ? null : { kind: "nested", tag, inside: literal };
  }

  if (literalTags.has(tag)) {
    const holder = stack.find((open) => formattingTags.has(open) || open === "a");

    return holder === undefined ? null : { kind: "nested", tag, inside: holder };
  }

  return tag === "blockquote" && stack.includes("blockquote")
    ? { kind: "nested", tag, inside: "blockquote" }
    : null;
}

// An entity Telegram reads: the four named ones and any numeric one. An ampersand that begins
// none of these is one the escaping missed.
const entity = /&(?:lt|gt|amp|quot|#\d+|#x[0-9a-f]+);/iy;

// The first fault in a piece of markup, or null when it has none. The tags are read for what they
// are and how they pair and nest, and the characters between them for whether the escaping held:
// a `<`, `>` or `&` standing outside a tag is one Telegram reads as markup it cannot parse. Whether
// the marks sit where they should is another question.
// A fault the code found, said in English: which tag, and where it was. A reason written from the
// markup itself is one a person can go back to the candidate and see, which is the whole
// difference between this and a sentence about what the markup looked like. A reader wanting it
// in another language reads the fault's kind and its data instead.
export function faultReason(fault: MarkupFault): string {
  switch (fault.kind) {
    case "unescaped":
      return `Structure check: the character ${fault.character} outside a tag is not escaped.`;
    case "unsupported":
      return `Structure check: the tag <${fault.tag}> is not supported.`;
    case "unclosed":
      return `Structure check: the tag <${fault.tag}> is never closed.`;
    case "unopened":
      return fault.open === null
        ? `Structure check: the tag </${fault.tag}> closes a tag that was never opened.`
        : `Structure check: the tag </${fault.tag}> closes while <${fault.open}> is still open.`;
    case "nested":
      return `Structure check: the tag <${fault.tag}> stands inside <${fault.inside}>, where it may not.`;
  }
}

export function markupFault(markup: string): MarkupFault | null {
  const stack: string[] = [];
  let index = 0;

  while (index < markup.length) {
    const character = markup[index];

    if (character === "&") {
      entity.lastIndex = index;

      if (!entity.test(markup)) {
        return { kind: "unescaped", character };
      }

      index = entity.lastIndex;
      continue;
    }

    if (character === ">") {
      return { kind: "unescaped", character };
    }

    if (character !== "<") {
      index += 1;
      continue;
    }

    tagAt.lastIndex = index;

    const token = tagAt.exec(markup);

    if (token === null) {
      return { kind: "unescaped", character };
    }

    index = tagAt.lastIndex;

    const tag = (token[2] ?? "").toLowerCase();
    const attributes = token[3] ?? "";
    const closing = token[1] === "/";

    if (!formattingTags.has(tag) && !literalTags.has(tag) && !otherTags.has(tag)) {
      return { kind: "unsupported", tag };
    }

    if (!closing && tag === "span" && !spoilerClass.test(attributes)) {
      return { kind: "unsupported", tag };
    }

    if (!closing) {
      const fault = nestingFault(tag, stack);

      if (fault !== null) {
        return fault;
      }

      stack.push(tag);
      continue;
    }

    if (stack[stack.length - 1] !== tag) {
      return { kind: "unopened", tag, open: stack[stack.length - 1] ?? null };
    }

    stack.pop();
  }

  const left = stack[stack.length - 1];

  return left === undefined ? null : { kind: "unclosed", tag: left };
}
