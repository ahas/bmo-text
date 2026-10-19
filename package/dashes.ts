// A dash left between words, replaced the way a person types the same sentence. The writer is
// told never to write one; this is the net under
// that rule on the way out, so it reads nothing but the characters: a name and its explanation get
// a colon, an aside between two dashes of one sentence gets commas (or brackets, where it has
// commas of its own), every further dash in the sentence a comma, and a dash between two digits is
// a range. What is someone else's or is not prose keeps its dashes: a code block or span, a quoted
// line, a web address, a dash standing alone in a table cell or at either end of a paragraph.
//
// A chat answer's line break is a line of its own on the screen, so a sentence ends with its line.
// A document wrapped at a fixed width runs one sentence over several lines, and is read with each
// paragraph or list item as one piece, its line breaks kept where they stand.

import { webAddresses } from "./web-address";

// An em dash or a horizontal bar, however many in a row, or an en dash with a space either side;
// an en dash touching its words is a range and stays.
const DASH = /[—―]+|(?<= )–(?= )/g;
const ANY_DASH = /[—―]| – /;
const FENCE = /^\s*```/;
const QUOTE = /^\s*>/;
// A line that opens a block of its own rather than running on from the one above it.
const BLOCK_START = /^\s*(?:$|[-*+•]\s|\d{1,3}[.)]\s|[①-⑳]|#|\||>|```)/;
// A block that is one line whatever follows it.
const ONE_LINE_BLOCK = /^\s*(?:#|\|)/;
// The end of a sentence: its closing mark, any footnote mark, emphasis, quote or bracket closing
// on it, then a space or the line's end.
const SENTENCE_END = /[.!?。](?=(?:\[[^\]\s]{1,8}\])*[*_"'”’)\]」』]*(?:\s|$))/g;
// A line's opening list mark, which is not a word the dash could stand after.
const LIST_MARK = /^\s*(?:[-*+•]|\d{1,3}[.)])$/;
// Marks after which the dash only doubled a pause the sentence already has.
const PAUSE = new Set([".", ",", ":", ";", "!", "?", "。", "、"]);
const OPENER = new Set(["(", "[", "{", "“", "‘", "「", "『"]);
const CLOSER = new Set([")", "]", "}", "”", "’", "」", "』"]);
// Marks inside an aside that commas around it would run together with.
const ASIDE_PAUSE = /[,;:]/;
// A label opening a line, such as "TITLE: " or "- Question: ": its colon names the line, and the
// sentence after it may still take one.
const LABEL = /^\s*(?:[-*+•]\s+|\d{1,3}[.)]\s+)?[^\s:.,;!?]+(?: [^\s:.,;!?]+){0,2}:(?=\s)/;

interface Dash {
  start: number;
  end: number;
  // The nearest character that is not white space on each side.
  left: number;
  right: number;
  sentence: number;
}

interface Swap {
  from: number;
  to: number;
  text: string;
}

const isSpace = (character: string | undefined): boolean => character === " " || character === "\t" || character === "\n";
const isDigit = (character: string | undefined): boolean => character !== undefined && character >= "0" && character <= "9";

// The stretches of a piece that are not the writer's prose: code spans, web addresses, and words in
// double quotes, which hold what somebody else said or wrote. A quote is paired within its line.
function keptSpans(piece: string): Array<[number, number]> {
  const spans: Array<[number, number]> = webAddresses(piece).map(({ start, end }) => [start, end]);
  let offset = 0;

  for (const line of piece.split("\n")) {
    for (const quoted of line.matchAll(/"[^"]*"|“[^”]*”/g)) {
      spans.push([offset + quoted.index, offset + quoted.index + quoted[0].length]);
    }

    offset += line.length + 1;
  }

  for (let at = 0; at < piece.length; ) {
    if (piece[at] !== "`") {
      at += 1;
      continue;
    }

    let run = 1;

    while (piece[at + run] === "`") run += 1;

    const close = piece.indexOf("`".repeat(run), at + run);

    if (close === -1) {
      at += run;
      continue;
    }

    spans.push([at, close + run]);
    at = close + run;
  }

  return spans;
}

// What takes a dash's place with the white space around it: a mark that holds on to the word
// before, a mark that holds on to the word after, and between them a space, or the line break and
// the indent of the line the text goes on in where the dash stood at a wrap.
function swapAt(piece: string, dash: Dash, before: string, after: string): Swap {
  const ahead = piece.slice(dash.left + 1, dash.start);
  const behind = piece.slice(dash.end, dash.right);
  const wrap = ahead.includes("\n") ? ahead.slice(ahead.indexOf("\n")) : behind.includes("\n") ? behind.slice(behind.indexOf("\n")) : " ";

  return { from: dash.left + 1, to: dash.right, text: `${before}${wrap}${after}` };
}

// Whether every bracket in a stretch closes inside it.
function balanced(stretch: string): boolean {
  let depth = 0;

  for (const character of stretch) {
    if (OPENER.has(character)) depth += 1;
    else if (CLOSER.has(character) && --depth < 0) return false;
  }

  return depth === 0;
}

function pieceWithoutDashes(piece: string): string {
  if (!ANY_DASH.test(piece)) return piece;

  const kept = keptSpans(piece);
  const inKept = (at: number): boolean => kept.some(([start, end]) => at >= start && at < end);
  const ends = [...piece.matchAll(SENTENCE_END)].map((found) => found.index).filter((at) => !inKept(at));
  const dashes: Dash[] = [];

  for (const found of piece.matchAll(DASH)) {
    const start = found.index;
    const end = start + found[0].length;

    if (inKept(start)) continue;

    let left = start - 1;
    let right = end;

    while (left >= 0 && isSpace(piece[left])) left -= 1;
    while (right < piece.length && isSpace(piece[right])) right += 1;

    if (left < 0 || right >= piece.length || piece[left] === "|" || piece[right] === "|") continue;
    // A dash a bracket or a quote holds is the character named, not a pause: "a dash (—)", " — ".
    if ((OPENER.has(piece[left] as string) || piece[left] === "\"") && (CLOSER.has(piece[right] as string) || piece[right] === "\"")) continue;
    if (LIST_MARK.test(piece.slice(piece.lastIndexOf("\n", left) + 1, left + 1))) continue;

    dashes.push({ start, end, left, right, sentence: ends.filter((at) => at < start).length });
  }

  const swaps: Swap[] = [];
  const prose: Dash[] = [];

  for (const dash of dashes) {
    const before = piece[dash.left] as string;

    if (dash.left === dash.start - 1 && dash.right === dash.end && isDigit(before) && isDigit(piece[dash.right])) {
      swaps.push({ from: dash.start, to: dash.end, text: "~" });
    } else if (PAUSE.has(before)) {
      swaps.push(swapAt(piece, dash, "", ""));
    } else if (OPENER.has(before)) {
      swaps.push({ from: dash.left + 1, to: dash.right, text: "" });
    } else {
      prose.push(dash);
    }
  }

  for (let at = 0; at < prose.length; ) {
    const dash = prose[at] as Dash;
    const next = prose[at + 1];

    // Two dashes set off an aside only where no bracket opens or closes between them without the
    // other: a dash inside one bracket and a dash inside the next are two dashes, not a pair.
    if (next !== undefined && next.sentence === dash.sentence && balanced(piece.slice(dash.right, next.left + 1))) {
      const bracketed = ASIDE_PAUSE.test(piece.slice(dash.right, next.left + 1));
      // Where the sentence already pauses right after the aside, its own mark closes it.
      const paused = PAUSE.has(piece[next.right] as string);
      const closing = paused
        ? { from: next.left + 1, to: next.right, text: bracketed ? ")" : "" }
        : swapAt(piece, next, bracketed ? ")" : ",", "");

      swaps.push(bracketed ? swapAt(piece, dash, "", "(") : swapAt(piece, dash, ",", ""), closing);
      at += 2;
      continue;
    }

    const lineStart = piece.lastIndexOf("\n", dash.start) + 1;
    const label = LABEL.exec(piece.slice(lineStart));
    const opening = Math.max(dash.sentence === 0 ? 0 : (ends[dash.sentence - 1] as number) + 1, label === null ? 0 : lineStart + label[0].length);
    const colonBefore =
      [...piece.slice(opening, dash.start)].some((character, offset) =>
        character === ":" && !inKept(opening + offset) && !(isDigit(piece[opening + offset - 1]) && isDigit(piece[opening + offset + 1]))) ||
      swaps.some((swap) => swap.text.startsWith(":") && swap.from >= opening && swap.from < dash.start);

    swaps.push(swapAt(piece, dash, colonBefore ? "," : ":", ""));
    at += 1;
  }

  swaps.sort((one, other) => one.from - other.from);

  let written = "";
  let from = 0;

  for (const swap of swaps) {
    written += piece.slice(from, swap.from) + swap.text;
    from = swap.to;
  }

  return written + piece.slice(from);
}

export function withoutDashes(text: string, options: { wrapped?: boolean } = {}): string {
  if (!ANY_DASH.test(text)) return text;

  // Each piece is read on its own: a line, or in a wrapped document a paragraph or a list item.
  // Code blocks and quoted lines are pieces left as they are.
  const pieces: Array<{ lines: string[]; prose: boolean }> = [];
  let fenced = false;

  for (const line of text.split("\n")) {
    const fence = FENCE.test(line);
    const prose = !fence && !fenced && !QUOTE.test(line) && line.trim() !== "";
    const last = pieces.at(-1);

    if (fence) fenced = !fenced;

    if (options.wrapped === true && prose && last?.prose === true && !BLOCK_START.test(line) && !ONE_LINE_BLOCK.test(last.lines[0] as string)) {
      last.lines.push(line);
    } else {
      pieces.push({ lines: [line], prose });
    }
  }

  return pieces.map((piece) => (piece.prose ? pieceWithoutDashes(piece.lines.join("\n")) : piece.lines.join("\n"))).join("\n");
}
