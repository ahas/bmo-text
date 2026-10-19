import { expandable } from "./format";

// What one Telegram message holds, counted in UTF-16 units after the markup is parsed, and plain words too
// long for one message divided as a client divides a long paste. A longer text is never cut and never
// refused: it is divided into consecutive messages, and the parts put back together are the text exactly.

// One message's text.
export const telegramTextLimit = 4096;
// What a file's caption holds on an account without Premium, which is the only size a sender can count on.
export const telegramCaptionLimit = 1024;
// The largest file the Bot API takes.
export const maxSendFileBytes = 50 * 1024 * 1024;

// The bot's own Telegram user id, which is the number before the colon of its token; nought where there is
// none.
export function botIdFromToken(token: string): number {
  const id = Number.parseInt(token.split(":")[0] ?? "", 10);

  return Number.isNaN(id) ? 0 : id;
}

const graphemes = new Intl.Segmenter("en", { granularity: "grapheme" });

function clusters(text: string): string[] {
  return [...graphemes.segment(text)].map((one) => one.segment);
}

// The furthest character boundary whose words come within the window.
function windowEnd(text: string, limit: number): number {
  let taken = 0;

  for (const character of text) {
    if (taken + character.length > limit) {
      return taken;
    }

    taken += character.length;
  }

  return text.length;
}

// Where the first part ends.
function cutPoint(text: string, limit: number): number {
  const window = text.slice(0, windowEnd(text, limit));

  for (const pause of ["\n\n", "\n", " "]) {
    const at = window.lastIndexOf(pause);

    if (at > 0) {
      return at + pause.length;
    }
  }

  let cut = 0;

  for (const cluster of clusters(text)) {
    if (cut + cluster.length > limit) {
      break;
    }

    cut += cluster.length;
  }

  // A single cluster longer than the window, which no real text carries, is divided between characters.
  return cut > 0 ? cut : Math.max(windowEnd(text, limit), [...text][0]?.length ?? 0);
}

// Plain words divided into consecutive messages of at most `limit` UTF-16 units each, the way a Telegram
// client sends a long paste: each part ends just after the last pause that fits (a blank line, else a line
// break, else a space), so the pause stays with the words before it, and where none fits, after the last
// whole character a reader sees as one.
export function plainParts(text: string, limit: number): string[] {
  const parts: string[] = [];
  let rest = text;

  while (rest.length > limit) {
    const cut = cutPoint(rest, limit);

    parts.push(rest.slice(0, cut));
    rest = rest.slice(cut);
  }

  if (rest !== "" || parts.length === 0) {
    parts.push(rest);
  }

  return parts;
}

// The reading a voice was spoken from, folded under the voice: its first part as the voice's caption, the
// rest as replies of one message each, every part an expandable quote of its escaped words. A part ends
// where the next whole character a reader sees as one would not fit, and only a character that alone exceeds
// a part is divided between its code points; nothing is dropped.
export function transcriptParts(text: string): string[] {
  const done: string[] = [];
  let part = "";
  let limit = telegramCaptionLimit;

  const append = (piece: string): void => {
    if (part.length + piece.length > limit) {
      if (part !== "") {
        done.push(part);
        part = "";
      }

      limit = done.length === 0 ? telegramCaptionLimit : telegramTextLimit;
    }

    part += piece;
  };

  for (const cluster of clusters(text)) {
    if (cluster.length > limit) {
      for (const point of cluster) {
        append(point);
      }
    } else {
      append(cluster);
    }
  }

  if (part !== "") {
    done.push(part);
  }

  return done.map(expandable);
}
