// A search finding's structure lines, turned into the markdown they stand for.
//
// This is apart from the renderer because two readers need it and only one of them may load the
// renderer: the renderer turns this markdown into HTML in the browser, and the screens turn the
// same markdown into a table of contents while the page is still being built. The renderer brings
// DOMPurify with it and is fetched only once a text is first shown, so a screen that imported it
// for the contents alone would pull the whole of it into the console's entry.
//
// Nothing here touches a DOM, and the marks it leaves — a heading per part, a heading per name —
// are what both readers count on standing in the same places.

import { plainHeading } from "./markdown-structure";

// The parts a label opens. `INTRO`, `BODY` and `CONCLUSION` are the parts an older finding was
// written in; no search writes them, and the findings that carry them still read.
export const PART_KEYS: readonly string[] = ["VERDICT", "SUMMARY", "INTRO", "BODY", "CONCLUSION", "UNCONFIRMED", "MORE"];

function isPart(key: string): boolean {
  return PART_KEYS.includes(key);
}

// The words a reader sees in a drawn finding, which are the consumer's to choose and read from its
// own language pack: what each part is called, by its key, and the label of each line an entry
// draws. `usual` holds `{usual}` where the usual price goes, and `noRating` is the word a search
// writes in a review's rating where the source gave none.
export interface FindingWords {
  parts: Readonly<Record<string, string>>;
  address: string;
  map: string;
  streetView: string;
  released: string;
  newer: string;
  productPage: string;
  stock: string;
  usual: string;
  deal: string;
  review: string;
  noRating: string;
  menu: string;
  dish: string;
  price: string;
  notes: string;
}

// The words a finding is drawn with where its consumer passes none: each part called by its key, and
// every line's label in English.
export const englishFindingWords: FindingWords = {
  parts: {},
  address: "Address",
  map: "Map",
  streetView: "Street view",
  released: "Released",
  newer: "Newest model",
  productPage: "Product page",
  stock: "Stock",
  usual: "usually {usual}",
  deal: "Deal",
  review: "Review",
  noRating: "none",
  menu: "Menu",
  dish: "Dish",
  price: "Price",
  notes: "Notes"
};

// What a reader calls a part: the consumer's word for it, else its key.
function partName(words: FindingWords, key: string): string {
  return words.parts[key] ?? key;
}

const STRUCTURE =
  /^[ \t]*(TITLE|NAME|VERDICT|SUMMARY|SECTION|INTRO|BODY|CONCLUSION|UNCONFIRMED|MORE|ADDRESS|MAP|ROADVIEW|MENU|RELEASED|NEWER|STOCK|DEAL|IMAGE|REVIEW|SOURCE(?:[ \t]*\[[^\]]*\])?|ANSWERED|TOPIC|KEYWORDS|RULE|KIND|DOCUMENT):[ \t]*(.*?)[ \t]*$/;

// A footnote mark in a finding's words — [3], or [3†] for a value read only in a search snippet —
// and the in-page link it becomes. A bracket followed by an address is a link of its own.
const FOOTNOTE = /\[(\d{1,3}†?)\](?![(:])/g;

export function footnoteMarks(line: string): string {
  return line.replace(FOOTNOTE, (_, mark: string) => `[${mark}](#mark-${encodeURIComponent(mark)})`);
}

// What one field line carries, cut at the separator the search writes between its parts.
function fields(value: string): string[] {
  return value.split(/[ \t]*::[ \t]*/);
}

// What a picture's line is captioned with where it is shown, out of the fields after its address: what
// it shows, then its shot and whose picture it is where the line names them. The entry, the third field, is
// the heading the picture already stands under.
export function pictureCaption(captioned: string): string {
  const [caption = "", , shot = "", source = ""] = fields(captioned.trim());

  return [caption, shot, source].filter((part) => part !== "").join(" · ");
}

// A heading the search wrote in its own words, and a fence line, outside of which one is read.
const AUTHORED = /^(#{1,6})([ \t].*)$/;
const FENCE = /^[ \t]*(```|~~~)/;

// The level the shallowest heading the search wrote itself stands at. The structure lines make the
// levels above it — a name the third, a section or a part the fourth — and a heading written inside
// a section is a part of that section, so it stands under it, and the rest keep their distance from
// it down to the sixth.
const AUTHORED_FROM = 5;

// The same for a text that draws no structure heading at all — a record or study material written
// in plain markdown. Nothing stands above its own headings, so they start where a name would, and a
// book's three levels keep three sizes instead of running into the sixth.
const AUTHORED_ALONE_FROM = 3;

// Whether a structure line is drawn as a heading: a name, a section that says what it holds, or a
// part label — every part but a verdict or a summary that carries its words on the same line.
function drawsHeading(key: string, value: string): boolean {
  return key === "TITLE" || key === "NAME" || (key === "SECTION" && value !== "") || (isPart(key) && !(inline(key) && value !== ""));
}

// The parts a result may write with their words on the label's own line, as a brief result does.
function inline(key: string): boolean {
  return key === "VERDICT" || key === "SUMMARY";
}

// How far the headings a finding wrote itself move down: from its shallowest to the fifth level
// under structure headings, to the third where there are none. A heading already deeper stays.
function authoredShift(lines: readonly string[]): number {
  let fenced = false;
  let shallowest = 7;
  let structured = false;

  for (const line of lines) {
    if (FENCE.test(line)) {
      fenced = !fenced;
    } else if (!fenced) {
      shallowest = Math.min(shallowest, AUTHORED.exec(line)?.[1]?.length ?? 7);

      const structure = STRUCTURE.exec(line);

      if (structure !== null && drawsHeading(structure[1] ?? "", structure[2] ?? "")) structured = true;
    }
  }

  return shallowest === 7 ? 0 : Math.max(0, (structured ? AUTHORED_FROM : AUTHORED_ALONE_FROM) - shallowest);
}

// The blocks a finding's structure lines make that markdown has no form for: a fold with its label,
// a gallery of pictures, and the map of a place. A fold and a gallery open on `::: <kind>` and close
// on a line of `:::` alone; a map is its one line, naming the place to find. The finding renderer
// draws them, and every other reader of this markdown takes them for paragraphs, which carry no
// heading. A line the search itself wrote in this form is escaped, so only a structure line opens one.
export const FINDING_BLOCK = /^::: (fold|gallery)(?: ([^\n]*))?\n([\s\S]*?)\n:::(?:\n|$)/;
export const FINDING_MAP = /^::: map ([^\n]+)(?:\n|$)/;

const ITEM = /^[ \t]*===ITEM===[ \t]*$/;
const IMAGE_LINE = /^[ \t]*IMAGE:/;
const BLANK = /^[ \t]*$/;

// The part of an entry a line stands in: an entry opens at a set's divider or at a name line, and
// what the map of its place is found by is read out of the whole of it, wherever in it the lines
// stand.
interface EntryPlace {
  address: string | null;
  name: string | null;
}

function entryPlaces(lines: readonly string[]): EntryPlace[] {
  const places: EntryPlace[] = [];
  let current: EntryPlace = { address: null, name: null };

  for (const line of lines) {
    const structure = STRUCTURE.exec(line);

    if (ITEM.test(line) || structure?.[1] === "NAME") {
      current = { address: null, name: null };
    }

    if (structure?.[1] === "ADDRESS" && current.address === null) current.address = structure[2] ?? null;
    if (structure?.[1] === "NAME") current.name = structure[2] ?? null;

    places.push(current);
  }

  return places;
}

// The words the map of a place is found by: the entry's address, else what its Naver Map link
// searches for, else the entry's name. Footnote marks are not part of any of them.
function mapQuery(link: string, place: EntryPlace | undefined): string | null {
  const address = place?.address?.replace(FOOTNOTE, "").trim() ?? "";

  if (address !== "") return address;

  const searched = /\/search\/([^/?#]+)/.exec(link)?.[1];

  if (searched !== undefined) {
    try {
      const words = decodeURIComponent(searched).trim();

      if (words !== "") return words;
    } catch {
      // A link whose search is not readable falls through to the name.
    }
  }

  const name = place?.name?.trim() ?? "";

  return name === "" ? null : name;
}

// A menu as a table: a dish and its price to a row, with a column for notes where a dish has one,
// and a line with no price a group row in bold. A cell keeps a bar the search wrote as a bar.
function menuTable(items: readonly string[], words: FindingWords): string[] {
  const rows = items.map((item) => fields(item.replace(/^[ \t]*(?:[-*+]|\d+\.)[ \t]+/, "").trim()).map((cell) => footnoteMarks(cell.replace(/\|/g, "\\|"))));
  const noted = rows.some((row) => row.length > 2);
  const columns = noted ? [words.dish, words.price, words.notes] : [words.dish, words.price];
  const cells = (row: readonly string[]): string => `| ${row.join(" | ")} |`;

  return [
    cells(columns),
    `| --- | --- |${noted ? " --- |" : ""}`,
    ...rows.map(([dish = "", price, ...notes]) =>
      price === undefined ? cells([`**${dish}**`, "", ...(noted ? [""] : [])]) : cells([dish, price, ...(noted ? [notes.join(" · ")] : [])])
    )
  ];
}

// A finding's structure lines as markdown. A picture is a paragraph of its own and a set's divider
// is a rule, so both are set apart by blank lines. Pictures written together are one gallery, a
// menu is a fold of its own, and a map line draws the place's map above its link.
export function findingMarkdown(text: string, words: FindingWords = englishFindingWords): string {
  const lines = text.split("\n");
  const shift = authoredShift(lines);
  const places = entryPlaces(lines);
  const out: string[] = [];
  let fenced = false;
  let at = 0;

  while (at < lines.length) {
    const line = lines[at]!;
    const index = at;

    at += 1;

    if (FENCE.test(line)) {
      fenced = !fenced;
      out.push(line);
      continue;
    }

    if (fenced) {
      out.push(line);
      continue;
    }

    // Pictures written one after another, blank lines between them allowed, are one gallery.
    if (IMAGE_LINE.test(line)) {
      const run = [line];
      let next = at;

      while (next < lines.length && (IMAGE_LINE.test(lines[next]!) || BLANK.test(lines[next]!))) {
        if (IMAGE_LINE.test(lines[next]!)) {
          run.push(lines[next]!);
          at = next + 1;
        }

        next += 1;
      }

      const pictures = run.map((picture) => structureLine("IMAGE", STRUCTURE.exec(picture)?.[2] ?? "", words)!.trim());

      out.push(pictures.length === 1 ? `\n${pictures[0]}\n` : `\n::: gallery\n${pictures.join("\n")}\n:::\n`);
      continue;
    }

    // A menu runs to the next structure line, divider, heading or fence, and is folded whole.
    const opening = STRUCTURE.exec(line);

    if (opening?.[1] === "MENU") {
      const items: string[] = [];

      while (at < lines.length && !STRUCTURE.test(lines[at]!) && !ITEM.test(lines[at]!) && !AUTHORED.test(lines[at]!) && !FENCE.test(lines[at]!)) {
        if (!BLANK.test(lines[at]!)) items.push(lines[at]!);
        at += 1;
      }

      const read = opening[2] ?? "";
      const body = [...(read === "" ? [] : [footnoteMarks(read), ""]), ...(items.length === 0 ? [] : menuTable(items, words))];

      if (body.length > 0) out.push(`\n::: fold ${words.menu}\n${body.join("\n")}\n:::\n`);
      continue;
    }

    if (opening?.[1] === "MAP") {
      const query = mapQuery(opening[2] ?? "", places[index]);
      const link = structureLine("MAP", opening[2] ?? "", words);

      out.push(query === null ? link! : `\n::: map ${query.replace(/\s+/g, " ")}\n\n${link}`);
      continue;
    }

    const drawn = lineMarkdown(line, shift, words);

    if (drawn !== null) out.push(drawn);
  }

  return out.join("\n");
}

// One line of a finding that stands on its own: a divider, a heading the search wrote, a line of
// its words, or a structure line. Null for a line that is not drawn.
function lineMarkdown(line: string, shift: number, words: FindingWords): string | null {
  if (ITEM.test(line)) {
    return "\n***\n";
  }

  // A line the search wrote in the form a finding block takes stays its own words.
  if (/^[ \t]*:::/.test(line)) {
    return footnoteMarks(line.replace(/^([ \t]*):/, "$1\\:"));
  }

  const authored = AUTHORED.exec(line);

  if (authored !== null) {
    return footnoteMarks(`${"#".repeat(Math.min(6, authored[1]!.length + shift))}${authored[2]}`);
  }

  const structure = STRUCTURE.exec(line);

  if (structure === null) {
    return footnoteMarks(line);
  }

  const [, key = "", value = ""] = structure;

  return structureLine(key, value, words);
}

// A structure line as the markdown it stands for, and null for one that is not drawn.
function structureLine(key: string, value: string, words: FindingWords): string | null {
  if (isPart(key) && value === "") {
    return `\n#### ${partName(words, key)}\n`;
  }

  switch (key) {
    case "TITLE":
    case "NAME":
      return `\n### ${value}\n`;
    // A section of the middle, headed by the name the search gave it for what it holds.
    case "SECTION":
      return value === "" ? null : `\n#### ${value}\n`;
    case "VERDICT":
    case "SUMMARY":
      return `\n**${partName(words, key)}** ${footnoteMarks(value)}`;
    case "INTRO":
    case "BODY":
    case "CONCLUSION":
    case "UNCONFIRMED":
    case "MORE":
      return `\n#### ${partName(words, key)}\n\n${footnoteMarks(value)}`;
    case "ADDRESS":
      return `**${words.address}** ${value}`;
    case "MAP":
      return `[${words.map}](<${value}>)`;
    case "ROADVIEW":
      return `[${words.streetView}](<${value}>)`;
    case "RELEASED":
      return `**${words.released}** ${footnoteMarks(value)}`;
    case "NEWER":
      return `**${words.newer}** ${footnoteMarks(value)}`;
    // What the product page said of itself, when it was read and where. The three mean
    // something only together: a selling state with no time on it is a state that was true
    // once, and the page is where a reader checks it again.
    case "STOCK": {
      const [state = "", read = "", page = ""] = fields(value);
      const when = read === "" ? "" : ` · ${read}`;
      const where = page === "" ? "" : ` [${words.productPage}](<${page}>)`;

      return `**${words.stock}** ${footnoteMarks(state)}${when}${where}`;
    }
    // An offer against what the thing usually costs, and the search's own word on whether
    // that gap is a gap.
    case "DEAL": {
      const [now = "", usual = "", real = ""] = fields(value);
      const against = usual === "" ? "" : ` (${words.usual.replace("{usual}", usual)})`;

      return `**${words.deal}** ${footnoteMarks(now)}${against}${real === "" ? "" : ` · ${footnoteMarks(real)}`}`;
    }
    case "IMAGE": {
      const [href = "", ...caption] = fields(value);

      return `\n![${pictureCaption(caption.join(" :: ")).replace(/[[\]]/g, "\\$&")}](<${href}>)\n`;
    }
    // One review as it was written: the rating the source gave it, the reviewer's own words
    // and the page they stand on, set as a quotation so it reads as somebody else's words
    // and not as the search's own. A field the result left out simply does not appear.
    case "REVIEW": {
      const [rating = "", said = "", page = ""] = fields(value);
      const scored = rating === "" || rating === words.noRating ? "" : `**${rating}** `;
      const where = page === "" ? "" : ` [${words.review}](<${page}>)`;

      return `\n> ${scored}${footnoteMarks(said)}${where}\n`;
    }
    default:
      return null;
  }
}

// Lines that are drawn and carry no prose: a table row, a rule, a picture, and a line opening a
// block of the finding's own.
const NOT_PROSE = /^[ \t]*(?:\||!\[|:::|([-*_])(?:[ \t]*\1){2,}[ \t]*$)/;
// What opens a line of a list or a quotation, which the words go on without.
const LEAD = /^[ \t]*(?:>[ \t]*)*(?:(?:[-*+]|\d{1,3}[.)])[ \t]+)?/;

// The prose lines of one finding, in order and as plain words. A part label carrying its words on
// its own line gives those words; every other structure line, a heading, a menu and whatever else is
// drawn rather than said gives none. What is folded under `MORE` is past the opening, so a finding's
// prose ends where its fold begins.
function proseLines(text: string): string[] {
  const prose: string[] = [];
  let fenced = false;
  let menu = false;

  for (const line of text.split("\n")) {
    if (FENCE.test(line)) {
      fenced = !fenced;
      continue;
    }

    const structure = fenced ? null : STRUCTURE.exec(line);

    if (menu && structure === null && !ITEM.test(line) && !AUTHORED.test(line)) continue;

    menu = structure?.[1] === "MENU";

    if (structure?.[1] === "MORE") break;
    if (fenced || BLANK.test(line) || ITEM.test(line) || AUTHORED.test(line) || NOT_PROSE.test(line)) continue;

    const [, key = "", value = ""] = structure ?? [];
    const words = structure === null ? line : isPart(key) ? value : "";
    const plain = plainHeading(words.replace(LEAD, "").replace(FOOTNOTE, "")).replace(/\s+/g, " ").trim();

    if (plain !== "") prose.push(plain);
  }

  return prose;
}

// The words a set of findings opens with, as one line of plain prose no longer than `limit`: what a
// place that shows a document by a few words and draws no markup is given, such as the card a chat
// app draws for a link to it. A line too long is cut at the last space before the limit and closed
// with an ellipsis. Empty where the findings say nothing in prose.
export function findingExcerpt(texts: readonly string[], limit: number): string {
  const lines: string[] = [];
  let length = 0;

  for (const line of texts.flatMap(proseLines)) {
    if (length > limit) break;

    lines.push(line);
    length += line.length + 1;
  }

  const joined = lines.join(" ");

  if (joined.length <= limit) return joined;

  const cut = joined.slice(0, limit - 1);
  const space = cut.lastIndexOf(" ");

  return `${(space > limit / 2 ? cut.slice(0, space) : cut).trimEnd()}…`;
}
