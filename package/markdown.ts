import { escapeAttribute, escapeHtml, expandableMarkup, isLinkable, longTextLength, markupFault, visibleText } from "./format";
import { bareAddressAt, gluedAfterAddress, markdownLinkAt } from "./web-address";
import { telegramTextLimit } from "./telegram-parts";

// An agent writes markdown and Telegram reads HTML. This module is the one place the first becomes the
// second: the markdown is read once, and the markup sent, the plain text held as its fallback and
// the blocks the fold and the division work on all come out of that one reading, so none of them
// can say something the others do not.
//
// Telegram shows a message line for line, so the reading is line-based rather than CommonMark's:
// a line break is a line break, an indented line is text, and a heading, a list, a table or a rule
// — none of which Telegram has a tag for — is laid flat into the text it holds. Only the tags the
// markup rulebook lists are ever built, and every character of the source is either a marker the
// construct it opened consumed or a character of text: a marker that pairs with nothing is text
// and gets no tag, so a marker and a tag never stand on the same place.

export type Inline =
  | { kind: "text"; text: string }
  | { kind: "code"; text: string }
  | { kind: "b" | "i" | "s" | "spoiler"; children: Inline[] }
  | { kind: "link"; url: string; children: Inline[] };

type Wrapper = Extract<Inline, { children: Inline[] }>;

export type Line = Inline[];

export type Block =
  | { kind: "lines"; lines: Line[] }
  | { kind: "pre"; language: string | null; text: string }
  | { kind: "quote"; blocks: Block[]; gaps: string[] };

// A document is its blocks and, before each, the line breaks that stood between it and the one
// before — so a message comes out spaced the way it was written. The first gap is always empty.
export interface MarkdownDocument {
  blocks: Block[];
  gaps: string[];
}

export interface Converted {
  source: string;
  document: MarkdownDocument;
  markup: string;
  plain: string;
  // Whether the markup says anything the plain text does not: a tag, or an escape the plain text
  // would not need. A message without it goes out as its plain text.
  formatted: boolean;
}

// ---------------------------------------------------------------------------------------------
// Inline

// The characters a backslash takes the meaning away from. Any other backslash is a character of
// the text, so a path or a regular expression survives as written.
const escapable = new Set(["`", "*", "~", "|", "[", "]", "\\"]);

// What may not stand glued to the outside of a pair: a letter, a digit, an underscore or a slash
// on the outside makes the marker part of a word or a path (2**3, server/**/*.ts), not markup.
const gluedOutside = /[A-Za-z0-9_/]/;

interface PairMarker {
  marker: string;
  kind: "b" | "i" | "s" | "spoiler";
}

const pairMarkers: PairMarker[] = [
  { marker: "**", kind: "b" },
  { marker: "~~", kind: "s" },
  { marker: "||", kind: "spoiler" },
  { marker: "*", kind: "i" },
];

function backtickRun(text: string, at: number): number {
  let end = at;

  while (text[end] === "`") {
    end += 1;
  }

  return end - at;
}

// Where the code span opening at `at` closes: a run of exactly as many backticks, or -1.
function codeClose(text: string, at: number, run: number): number {
  let from = at + run;

  while (from < text.length) {
    const next = text.indexOf("`", from);

    if (next === -1) {
      return -1;
    }

    const length = backtickRun(text, next);

    if (length === run) {
      return next;
    }

    from = next + length;
  }

  return -1;
}

function opensPair(text: string, at: number, marker: string): boolean {
  const character = marker[0] ?? "";
  const before = text[at - 1] ?? "";
  const after = text[at + marker.length] ?? "";

  return (
    text.startsWith(marker, at) &&
    !gluedOutside.test(before) &&
    before !== character &&
    after !== "" &&
    !/\s/.test(after) &&
    after !== character
  );
}

function closesPair(text: string, at: number, marker: string): boolean {
  const character = marker[0] ?? "";
  const before = text[at - 1] ?? "";
  const after = text[at + marker.length] ?? "";

  return (
    text.startsWith(marker, at) &&
    before !== "" &&
    !/\s/.test(before) &&
    before !== character &&
    !gluedOutside.test(after) &&
    after !== character
  );
}

// Where the pair opening at `at` closes, looking past code spans and escapes, which hold no
// markers of their own. -1 when nothing closes it on this line.
function pairClose(text: string, at: number, marker: string): number {
  let index = at + marker.length + 1;

  while (index < text.length) {
    const character = text[index];

    if (character === "\\" && escapable.has(text[index + 1] ?? "")) {
      index += 2;
      continue;
    }

    if (character === "`") {
      const run = backtickRun(text, index);
      const close = codeClose(text, index, run);

      index = close === -1 ? index + run : close + run;
      continue;
    }

    if (closesPair(text, index, marker)) {
      return index;
    }

    index += 1;
  }

  return -1;
}

const autolink = /^<(https?:\/\/[^\s<>]+)>/i;

function pushText(nodes: Inline[], text: string): void {
  if (text === "") {
    return;
  }

  const last = nodes[nodes.length - 1];

  if (last?.kind === "text") {
    last.text += text;
  } else {
    nodes.push({ kind: "text", text });
  }
}

// Where a construct stood in the line it was read from: the code spans and links, which are
// literal to the end, and the formatting pairs. A division or a table cell that cut through one
// would leave both halves holding a marker with nothing to pair with.
interface InlineSpan {
  start: number;
  end: number;
  literal: boolean;
}

// One line of markdown as inline nodes. Nothing here crosses a line break: the caller hands over
// one line at a time. spans, when given, collects where each construct of the line stood.
export function parseInline(text: string, spans?: InlineSpan[]): Inline[] {
  const nodes: Inline[] = [];
  let index = 0;

  while (index < text.length) {
    const character = text[index] ?? "";

    if (character === "\\" && escapable.has(text[index + 1] ?? "")) {
      pushText(nodes, text[index + 1] ?? "");
      index += 2;
      continue;
    }

    if (character === "`") {
      const run = backtickRun(text, index);
      const close = codeClose(text, index, run);

      if (close === -1) {
        pushText(nodes, "`".repeat(run));
        index += run;
        continue;
      }

      let inner = text.slice(index + run, close);

      if (inner.length > 1 && inner.startsWith(" ") && inner.endsWith(" ") && inner.trim() !== "") {
        inner = inner.slice(1, -1);
      }

      nodes.push({ kind: "code", text: inner });
      spans?.push({ start: index, end: close + run, literal: true });
      index = close + run;
      continue;
    }

    const image = character === "!" && text[index + 1] === "[";

    if (character === "[" || image) {
      const found = markdownLinkAt(text, image ? index + 1 : index);

      if (found !== null) {
        const label = image ? [{ kind: "text", text: found.label } as Inline] : parseInline(found.label);
        const shown: Inline[] = label.length === 0 ? [{ kind: "text", text: found.url }] : label;

        if (isLinkable(found.url)) {
          nodes.push({ kind: "link", url: found.url, children: shown });
        } else {
          for (const node of shown) {
            node.kind === "text" ? pushText(nodes, node.text) : nodes.push(node);
          }

          pushText(nodes, ` ${found.url}`);
        }

        spans?.push({ start: index, end: found.end, literal: true });
        index = found.end;
        continue;
      }
    }

    if (character === "<") {
      const found = autolink.exec(text.slice(index));

      if (found !== null) {
        pushText(nodes, found[1] ?? "");
        spans?.push({ start: index, end: index + found[0].length, literal: true });
        index += found[0].length;
        continue;
      }
    }

    // A bare address stays as characters for Telegram to show as a link on its own, except where
    // the sentence runs on straight after it, a particle glued to it or to its closing bracket,
    // which Telegram would read as part of the address. There it is a link of its own, ending where
    // the sentence takes over.
    if (character === "h" || character === "H") {
      const url = bareAddressAt(text, index);

      if (url !== null && gluedAfterAddress(text, index + url.length)) {
        nodes.push({ kind: "link", url, children: [{ kind: "text", text: url }] });
        spans?.push({ start: index, end: index + url.length, literal: true });
        index += url.length;
        continue;
      }
    }

    const pair = pairMarkers.find((one) => opensPair(text, index, one.marker));

    if (pair !== undefined) {
      const close = pairClose(text, index, pair.marker);

      if (close !== -1) {
        nodes.push({ kind: pair.kind, children: parseInline(text.slice(index + pair.marker.length, close)) });
        spans?.push({ start: index, end: close + pair.marker.length, literal: false });
        index = close + pair.marker.length;
        continue;
      }
    }

    // A run of the same marker character that opened nothing stays text as one piece, so its tail
    // is not read as a marker of its own.
    if (character === "*" || character === "~" || character === "|") {
      let end = index;

      while (text[end] === character) {
        end += 1;
      }

      pushText(nodes, text.slice(index, end));
      index = end;
      continue;
    }

    pushText(nodes, character);
    index += 1;
  }

  return nodes;
}

// ---------------------------------------------------------------------------------------------
// Blocks

const fenceOpen = /^ {0,3}(`{3,}|~{3,})[ \t]*([^\s`]*)[^`]*$/;
const fenceClose = /^ {0,3}(`{3,}|~{3,})[ \t]*$/;
const languageName = /^[A-Za-z0-9_+#.-]{1,32}$/;
const headingLine = /^ {0,3}#{1,6}(?:[ \t]+(.*?))?(?:[ \t]+#+)?[ \t]*$/;
const ruleLine = /^ {0,3}([-*_])(?:[ \t]*\1){2,}[ \t]*$/;
const tableRule = /^[ \t]*\|?[ \t]*:?-+:?[ \t]*(?:\|[ \t]*:?-+:?[ \t]*)*\|?[ \t]*$/;
const bulletLine = /^([ \t]*)[-*+][ \t]+(.*)$/;
const numberedLine = /^([ \t]*\d{1,3}[.)][ \t]+)(.*)$/;
const quoteLine = /^ {0,3}>/;
const quoteMarks = /^ {0,3}(?:>[ \t]?)+/;

// The cells of a table row. A pipe is a cell border only where it stands outside a code span or a
// link — a shell pipe in a command, a query string in an address — and is not escaped.
function tableCells(row: string): string[] {
  let body = row.trim();

  if (body.startsWith("|")) {
    body = body.slice(1);
  }

  if (body.endsWith("|") && !body.endsWith("\\|")) {
    body = body.slice(0, -1);
  }

  const spans: InlineSpan[] = [];

  parseInline(body, spans);

  const cells: string[] = [];
  let from = 0;

  for (let index = 0; index < body.length; index += 1) {
    const inside = spans.some((span) => span.literal && span.start < index && index < span.end);

    if (body[index] === "|" && body[index - 1] !== "\\" && !inside) {
      cells.push(body.slice(from, index));
      from = index + 1;
    }
  }

  cells.push(body.slice(from));

  return cells.map((cell) => cell.trim());
}

function isTableStart(lines: string[], at: number): boolean {
  const header = lines[at] ?? "";
  const rule = lines[at + 1] ?? "";

  return header.includes("|") && rule.includes("|") && tableRule.test(rule);
}

function tableRow(row: string): Line {
  const line: Line = [];

  for (const [index, cell] of tableCells(row).entries()) {
    if (index > 0) {
      pushText(line, " · ");
    }

    for (const node of parseInline(cell)) {
      node.kind === "text" ? pushText(line, node.text) : line.push(node);
    }
  }

  return line;
}

// A line of text laid flat: a heading loses its marks, a bullet becomes •, a number stays as it
// was written. null is a line that is not text at all — a rule, or a heading with nothing in it.
function flatLine(line: string): Line | null {
  if (ruleLine.test(line)) {
    return null;
  }

  const heading = headingLine.exec(line);

  if (heading !== null) {
    const words = heading[1] ?? "";

    return words.trim() === "" ? null : parseInline(words);
  }

  const bullet = bulletLine.exec(line);

  if (bullet !== null) {
    return prefixed(`${bullet[1] ?? ""}• `, bullet[2] ?? "");
  }

  const numbered = numberedLine.exec(line);

  if (numbered !== null) {
    return prefixed(numbered[1] ?? "", numbered[2] ?? "");
  }

  return parseInline(line);
}

function prefixed(prefix: string, rest: string): Line {
  const line: Line = [{ kind: "text", text: prefix }];

  for (const node of parseInline(rest)) {
    node.kind === "text" ? pushText(line, node.text) : line.push(node);
  }

  return line;
}

function parseBlocks(lines: string[], quotes: boolean): MarkdownDocument {
  const blocks: Block[] = [];
  const gaps: string[] = [];
  let current: Line[] = [];
  // Line breaks owed before the next block: one ends the line before it, and every blank line
  // adds one more.
  let pending = 0;
  let index = 0;

  const push = (block: Block): void => {
    gaps.push(blocks.length === 0 ? "" : "\n".repeat(Math.max(1, pending)));
    blocks.push(block);
    pending = 0;
  };

  const flush = (): void => {
    if (current.length > 0) {
      push({ kind: "lines", lines: current });
      current = [];
      pending = 1;
    }
  };

  while (index < lines.length) {
    const line = lines[index] ?? "";

    if (line.trim() === "") {
      flush();
      pending += 1;
      index += 1;
      continue;
    }

    const fence = fenceOpen.exec(line);

    if (fence !== null) {
      flush();

      const opener = fence[1] ?? "```";
      const body: string[] = [];

      index += 1;

      while (index < lines.length) {
        const closing = fenceClose.exec(lines[index] ?? "");

        if (closing !== null && (closing[1] ?? "")[0] === opener[0] && (closing[1] ?? "").length >= opener.length) {
          break;
        }

        body.push(lines[index] ?? "");
        index += 1;
      }

      index += 1;

      const language = fence[2] ?? "";

      // An empty code block is a tag around nothing, which Telegram has no use for.
      if (body.join("").trim() !== "") {
        push({ kind: "pre", language: languageName.test(language) ? language : null, text: body.join("\n") });
        pending = 1;
      }

      continue;
    }

    if (quotes && quoteLine.test(line)) {
      flush();

      const inner: string[] = [];

      while (index < lines.length && quoteLine.test(lines[index] ?? "")) {
        inner.push((lines[index] ?? "").replace(quoteMarks, ""));
        index += 1;
      }

      const quoted = parseBlocks(inner, false);

      if (quoted.blocks.length > 0) {
        push({ kind: "quote", blocks: quoted.blocks, gaps: quoted.gaps });
        pending = 1;
      }

      continue;
    }

    if (isTableStart(lines, index)) {
      current.push(tableRow(line));
      index += 2;

      while (index < lines.length && (lines[index] ?? "").includes("|") && (lines[index] ?? "").trim() !== "") {
        current.push(tableRow(lines[index] ?? ""));
        index += 1;
      }

      continue;
    }

    const flat = flatLine(line);

    if (flat !== null) {
      current.push(flat);
    } else if (current.length === 0) {
      // A rule standing between blank lines takes one of them with it, so the text on either side
      // is one paragraph break apart rather than two.
      pending = Math.max(0, pending - 1);
    }

    index += 1;
  }

  flush();

  return { blocks, gaps };
}

export function parseMarkdown(source: string): MarkdownDocument {
  return parseBlocks(source.replaceAll("\r\n", "\n").split("\n"), true);
}

// ---------------------------------------------------------------------------------------------
// Rendering

// A code span inside a link keeps its letters and loses its tag, since a link may not hold one.
interface Leaf {
  kind: "text" | "code";
  text: string;
  wrappers: Wrapper[];
}

function leaves(nodes: Inline[], wrappers: Wrapper[], into: Leaf[]): Leaf[] {
  for (const node of nodes) {
    if (node.kind === "text" || node.kind === "code") {
      const inLink = wrappers.some((one) => one.kind === "link");

      into.push({ kind: node.kind === "code" && !inLink ? "code" : "text", text: node.text, wrappers });
    } else {
      leaves(node.children, [...wrappers, node], into);
    }
  }

  return into;
}

function openTag(wrapper: Wrapper): string {
  switch (wrapper.kind) {
    case "link":
      return `<a href="${escapeAttribute(wrapper.url)}">`;
    case "spoiler":
      return "<tg-spoiler>";
    default:
      return `<${wrapper.kind}>`;
  }
}

function closeTag(wrapper: Wrapper): string {
  return wrapper.kind === "link" ? "</a>" : wrapper.kind === "spoiler" ? "</tg-spoiler>" : `</${wrapper.kind}>`;
}

// A line's markup, built from its leaves with a stack of open tags. A code leaf closes every tag
// around it and reopens them after, which is what splits a formatting around the code it held:
// Telegram refuses a code inside a bold, and the writer's literal is the one thing not to lose.
// Opening and closing by the stack is also what keeps every tag paired and properly nested.
export function renderInline(nodes: Inline[]): string {
  const open: Wrapper[] = [];
  let out = "";

  for (const leaf of leaves(nodes, [], [])) {
    if (leaf.text === "") {
      continue;
    }

    const wanted = leaf.kind === "code" ? [] : leaf.wrappers;
    let shared = 0;

    while (shared < open.length && shared < wanted.length && open[shared] === wanted[shared]) {
      shared += 1;
    }

    while (open.length > shared) {
      out += closeTag(open.pop() as Wrapper);
    }

    for (const wrapper of wanted.slice(shared)) {
      out += openTag(wrapper);
      open.push(wrapper);
    }

    out += leaf.kind === "code" ? `<code>${escapeHtml(leaf.text)}</code>` : escapeHtml(leaf.text);
  }

  while (open.length > 0) {
    out += closeTag(open.pop() as Wrapper);
  }

  return out;
}

export function plainInline(nodes: Inline[]): string {
  return nodes
    .map((node) => {
      switch (node.kind) {
        case "text":
        case "code":
          return node.text;
        case "link": {
          const label = plainInline(node.children);

          return label === node.url ? label : `${label} (${node.url})`;
        }
        default:
          return plainInline(node.children);
      }
    })
    .join("");
}

// quoted is false inside the folded part: an expandable quote cannot hold a quote, so a quote there
// stands as its own lines.
function renderBlock(block: Block, quoted = true): string {
  switch (block.kind) {
    case "lines":
      return block.lines.map(renderInline).join("\n");
    case "pre": {
      const body = escapeHtml(block.text);

      return block.language === null ? `<pre>${body}</pre>` : `<pre><code class="language-${block.language}">${body}</code></pre>`;
    }
    case "quote": {
      const inner = renderBlocks(block.blocks, block.gaps);

      return quoted ? `<blockquote>${inner}</blockquote>` : inner;
    }
  }
}

function renderBlocks(blocks: Block[], gaps: string[], quoted = true): string {
  return blocks.map((block, index) => (gaps[index] ?? "") + renderBlock(block, quoted)).join("");
}

function plainBlock(block: Block): string {
  switch (block.kind) {
    case "lines":
      return block.lines.map(plainInline).join("\n");
    case "pre":
      return block.text;
    case "quote":
      return plainBlocks(block.blocks, block.gaps);
  }
}

function plainBlocks(blocks: Block[], gaps: string[]): string {
  return blocks.map((block, index) => (gaps[index] ?? "") + plainBlock(block)).join("");
}

// ---------------------------------------------------------------------------------------------
// The one-bold rule

// A line's content once a leading bullet, number or circled number and the spaces around it are
// set aside: what an item name is measured against.
const itemPrefix = /^[ \t]*(?:•|\d{1,3}[.)]|[①-⑳])?[ \t]*/;

function isWholeLine(line: Line, node: Inline): boolean {
  const others = line.filter((one) => one !== node);

  return (
    line.includes(node) &&
    others.every((one) => one.kind === "text") &&
    others.map((one) => (one as { text: string }).text).join("").replace(itemPrefix, "").trim() === "" &&
    (line.indexOf(node) === 0 || line.slice(0, line.indexOf(node)).every((one) => one.kind === "text"))
  );
}

interface BoldPlace {
  node: Wrapper;
  line: Line;
  parent: Inline[];
}

function boldsIn(nodes: Inline[], line: Line, into: BoldPlace[]): void {
  for (const node of nodes) {
    if (node.kind === "b") {
      into.push({ node, line, parent: nodes });
    } else if (node.kind !== "text" && node.kind !== "code") {
      boldsIn(node.children, line, into);
    }
  }
}

function linesOf(blocks: Block[]): Line[] {
  return blocks.flatMap((block) =>
    block.kind === "lines" ? block.lines : block.kind === "quote" ? linesOf(block.blocks) : [],
  );
}

// Bold is one per message, except the item names of a list. What makes a bold an item name is
// where it stands: several survive only when every one of them is a whole line, and otherwise the
// first stands alone and the rest keep their words without the bold.
function limitBold(document: MarkdownDocument): void {
  const bolds: BoldPlace[] = [];

  for (const line of linesOf(document.blocks)) {
    boldsIn(line, line, bolds);
  }

  if (bolds.length <= 1 || bolds.every((one) => isWholeLine(one.line, one.node))) {
    return;
  }

  for (const one of bolds.slice(1)) {
    const at = one.parent.indexOf(one.node);

    if (at !== -1) {
      one.parent.splice(at, 1, ...one.node.children);
    }
  }
}

// ---------------------------------------------------------------------------------------------
// Conversion

export function convertMarkdown(source: string): Converted {
  const document = parseMarkdown(source);
  limitBold(document);
  const markup = renderBlocks(document.blocks, document.gaps);
  const plain = plainBlocks(document.blocks, document.gaps);

  return { source, document, markup, plain, formatted: markup !== escapeHtml(plain) };
}

// ---------------------------------------------------------------------------------------------
// The fold

const foldMaxOpeningLength = 400;
const foldMinHiddenLength = 80;
const foldMinHiddenLines = 3;
// A last block this short is the few lines that tie the message together and never folds; a
// longer one is body, not conclusion. The bound is generous on purpose: an open block of body
// costs a little screen, a hidden conclusion costs the answer.
const foldMaxClosingLength = 200;
const foldMaxClosingLines = 3;
const foldMinLines = 8;

// A sentence that ends. A conclusion is one; the last line of a list is a name or a phrase and is
// not, which is what keeps a list from having its last item lifted out of the quote alone.
const finishedSentence = /[.!?…]["'”’)\]」』]?$/;

function isConclusion(text: string): boolean {
  return text.length <= foldMaxClosingLength && text.split("\n").length <= foldMaxClosingLines;
}

// Where a fold falls, decided on blocks: the opening and the closing stay in the open and the
// hidden blocks go behind the quote. A code block or a quote is one block whatever blank lines it
// holds, so the fold never cuts through one, and neither is ever the conclusion.
export interface Fold {
  opening: Block | null;
  // The line breaks between the opening and the quote.
  openingGap: string;
  hidden: Block[];
  // The line breaks before each hidden block, the first of them empty.
  hiddenGaps: string[];
  closing: Block | null;
  closingGap: string;
}

export function foldBlocks(document: MarkdownDocument): Fold | null {
  let body = [...document.blocks];
  let bodyGaps = [...document.gaps];
  let closing: Block | null = null;
  let closingGap = "";
  const last = body[body.length - 1];

  if (last?.kind === "lines" && body.length > 1 && isConclusion(plainBlock(last))) {
    closing = last;
    closingGap = bodyGaps[bodyGaps.length - 1] ?? "\n\n";
    body = body.slice(0, -1);
    bodyGaps = bodyGaps.slice(0, -1);
  } else if (last?.kind === "lines" && last.lines.length > 1) {
    const tail = last.lines[last.lines.length - 1] ?? [];
    const tailText = plainInline(tail).trim();
    const rest = last.lines.slice(0, -1);

    if (plainBlock({ kind: "lines", lines: rest }).trim() !== "" && isConclusion(tailText) && finishedSentence.test(tailText)) {
      closing = { kind: "lines", lines: [tail] };
      closingGap = "\n";
      body = [...body.slice(0, -1), { kind: "lines", lines: rest }];
    }
  }

  const first = body[0];
  const opening = body.length > 1 && first !== undefined && plainBlock(first).length <= foldMaxOpeningLength ? first : null;
  const hidden = opening === null ? body : body.slice(1);
  const hiddenGaps = ["", ...bodyGaps.slice(opening === null ? 1 : 2)];
  const hiddenText = plainBlocks(hidden, hiddenGaps);

  if (
    hidden.length === 0 ||
    hiddenText.trim() === "" ||
    (hiddenText.length < foldMinHiddenLength && hiddenText.split("\n").length < foldMinHiddenLines)
  ) {
    return null;
  }

  return { opening, openingGap: bodyGaps[1] ?? "\n\n", hidden, hiddenGaps, closing, closingGap };
}

// The message folded, or null where there is too little to hide for a fold to help.
export function foldedMarkup(converted: Converted): string | null {
  const fold = foldBlocks(converted.document);

  if (fold === null) {
    return null;
  }

  const opened = fold.opening === null ? "" : renderBlock(fold.opening) + fold.openingGap;
  const quoted = expandableMarkup(renderBlocks(fold.hidden, fold.hiddenGaps, false));
  const closed = fold.closing === null ? "" : fold.closingGap + renderBlock(fold.closing);

  return opened + quoted + closed;
}

// What a send path that asks no gate puts up: the fold the code makes when the message is long
// enough for one, the converted markup when it carries formatting, and null — the plain text —
// otherwise. Markup the structure check refuses is never it.
export function codeMarkup(converted: Converted): string | null {
  const long = converted.plain.length > longTextLength || converted.plain.split("\n").length > foldMinLines;
  const folded = long ? foldedMarkup(converted) : null;
  const chosen = folded ?? (converted.formatted ? converted.markup : null);

  return chosen !== null && markupFault(chosen) === null ? chosen : null;
}

// ---------------------------------------------------------------------------------------------
// Division

// How long a piece of markdown is to Telegram: the longer of what its markup shows and its plain
// text, since either may be the one that goes out.
function sentLength(source: string): number {
  const converted = convertMarkdown(source);

  return Math.max(visibleText(converted.markup).length, converted.plain.length);
}

// Whether the source up to `at` leaves a fence open, and its opening line if so.
function openFence(source: string, at: number): string | null {
  let open: string | null = null;
  let marker = "";

  for (const line of source.slice(0, at).split("\n")) {
    if (open === null) {
      const found = fenceOpen.exec(line);

      if (found !== null) {
        open = line.trimEnd();
        marker = found[1] ?? "```";
      }
    } else {
      const closing = fenceClose.exec(line);

      if (closing !== null && (closing[1] ?? "")[0] === marker[0] && (closing[1] ?? "").length >= marker.length) {
        open = null;
      }
    }
  }

  return open;
}

// Where a part ends inside the window. A line break is taken before a space, and a space only where
// it stands outside every construct of its line — a code span or a bold cut through its middle
// would leave a stray marker at the end of one part and another at the start of the next. Only a
// line whose constructs leave no such space inside the window is cut at a space regardless.
function cutPoint(text: string, window: number): number {
  const head = text.slice(0, window);

  for (const pause of ["\n\n", "\n"]) {
    const at = head.lastIndexOf(pause);

    if (at > 0) {
      return at + pause.length;
    }
  }

  const spans: InlineSpan[] = [];

  parseInline(text.slice(0, text.indexOf("\n") === -1 ? text.length : text.indexOf("\n")), spans);

  for (let at = head.lastIndexOf(" "); at > 0; at = head.lastIndexOf(" ", at - 1)) {
    if (!spans.some((span) => span.start < at && at < span.end)) {
      return at + 1;
    }
  }

  const loose = head.lastIndexOf(" ");

  return loose > 0 ? loose + 1 : window;
}

// A longer text is divided into consecutive messages the way a reader pauses — the last blank
// line, else the last line break, else the last space inside the window — and each part is a whole
// piece of markdown: a code block cut in two is closed at the end of one part and opened again,
// with its language, at the start of the next. A part is measured as Telegram will count it once
// converted, and cut shorter where that is still too long. The parts put back together are the
// text, save for the fence lines a divided code block gains.
export function markdownParts(text: string, limit: number = telegramTextLimit): string[] {
  const parts: string[] = [];
  let rest = text;

  while (rest.length > 0 && sentLength(rest) > limit) {
    let window = Math.min(rest.length, limit);
    let part = "";
    let next = "";

    for (;;) {
      const cut = Math.max(1, cutPoint(rest, window));
      const fence = openFence(rest, cut);

      part = rest.slice(0, cut);
      next = rest.slice(cut);

      if (fence !== null) {
        const marker = /`{3,}|~{3,}/.exec(fence)?.[0] ?? "```";

        part = `${part.endsWith("\n") ? part : `${part}\n`}${marker}`;
        next = `${fence}\n${next}`;
      }

      if (sentLength(part) <= limit || window <= 1) {
        break;
      }

      window = Math.floor(window * 0.9);
    }

    // A division that gave back as much as it took — a fence reopened after a cut too short to
    // carry it — would never end, so it falls back to cutting the window as it stands.
    if (next.length >= rest.length) {
      part = rest.slice(0, limit);
      next = rest.slice(limit);
    }

    parts.push(part);
    rest = next;
  }

  if (rest !== "" || parts.length === 0) {
    parts.push(rest);
  }

  return parts;
}
