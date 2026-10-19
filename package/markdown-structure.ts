// The structure of a markdown text, read off its source: its headings, and where it may be cut.
//
// The console's document viewer makes its
// table of contents out of them and stamps the same marks on the headings it draws, the search
// documents page does the same for its findings, the console cuts a document into parts for the
// translator at them, and the bot counts them, with the paragraphs and the inline code, to tell a
// translation that kept its structure from one that lost it. All of them read the headings here, so
// a heading is one thing to all of them.
//
// Only ATX headings count — a line of one to six `#`. A line inside a fenced code block is code,
// whatever it starts with, and no text is cut inside one.

export interface MarkdownHeading {
  // 1 for `#`, 6 for `######`.
  depth: number;
  // What the heading says, with its inline markup taken off.
  text: string;
  // A mark no other heading of the same text carries, made from its words: the first of a text
  // takes the words alone, the ones after it a number as well.
  slug: string;
  // The source line the heading stands on, counted from 1.
  line: number;
}

const ATX = /^ {0,3}(#{1,6})(?:[ \t]+(.*?))?(?:[ \t]+#+)?[ \t]*$/;
const FENCE = /^ {0,3}(`{3,}|~{3,})/;

// A heading's words without the markup around them: a link or an image stands as its words, and
// emphasis, strikes and code marks fall away.
export function plainHeading(text: string): string {
  return text
    .replace(/!?\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/`+/g, "")
    .replace(/(\*\*|__|~~)(.+?)\1/g, "$2")
    .replace(/(^|\W)[*_](\S(?:.*?\S)?)[*_](?=\W|$)/g, "$1$2")
    .replace(/\\([\\`*_{}[\]()#+\-.!~])/g, "$1")
    .trim();
}

// The words of a heading as a mark: lower-case, letters and digits of any script kept, runs of
// space made one hyphen, everything else dropped.
export function headingSlug(text: string): string {
  const slug = text
    .normalize("NFC")
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s_-]/gu, "")
    .trim()
    .replace(/\s+/g, "-");

  return slug === "" ? "section" : slug;
}

// Every heading of a text, in the order it stands. A heading with no words is no entry.
export function markdownHeadings(source: string): MarkdownHeading[] {
  const headings: MarkdownHeading[] = [];
  const taken = new Set<string>();
  const counts = new Map<string, number>();
  let fence: string | null = null;
  const lines = source.replace(/\r\n?/g, "\n").split("\n");

  for (const [index, line] of lines.entries()) {
    const opening = FENCE.exec(line);

    if (fence !== null) {
      if (opening !== null && opening[1]![0] === fence[0] && opening[1]!.length >= fence.length && line.trim() === opening[1]) {
        fence = null;
      }

      continue;
    }

    if (opening !== null) {
      fence = opening[1]!;
      continue;
    }

    const atx = ATX.exec(line);

    if (atx === null) {
      continue;
    }

    const text = plainHeading(atx[2] ?? "");

    if (text === "") {
      continue;
    }

    const base = headingSlug(text);
    let slug = base;
    let count = counts.get(base) ?? 0;

    while (taken.has(slug)) {
      count += 1;
      slug = `${base}-${count}`;
    }

    counts.set(base, count);
    taken.add(slug);
    headings.push({ depth: atx[1]!.length, text, slug, line: index + 1 });
  }

  return headings;
}

// The fence a line opens or closes a code block with, as far as a line alone can say.
function fenceOf(line: string): string | null {
  return FENCE.exec(line)?.[1] ?? null;
}

// Where a text may be cut in two: after a blank line outside any code block, or failing that at a
// line break outside one. Each is the offset the second half starts at.
function cutPoints(text: string): { paragraphs: number[]; lines: number[] } {
  const paragraphs: number[] = [];
  const lines: number[] = [];
  let fence: string | null = null;
  let offset = 0;
  const all = text.split("\n");

  for (const [index, line] of all.entries()) {
    const marker = fenceOf(line);

    if (fence !== null) {
      if (marker !== null && marker[0] === fence[0] && marker.length >= fence.length && line.trim() === marker) {
        fence = null;
      }
    } else if (marker !== null) {
      fence = marker;
    }

    offset += line.length + 1;

    if (fence === null && index < all.length - 1 && offset < text.length) {
      lines.push(offset);

      if (line.trim() === "") {
        paragraphs.push(offset);
      }
    }
  }

  return { paragraphs, lines };
}

// A text in two, at the paragraph boundary nearest its middle, or the line boundary nearest it
// where there is no paragraph boundary, with what joins the halves back; null where it is one line.
export function halveText(text: string): { halves: [string, string]; joint: string } | null {
  const { paragraphs, lines } = cutPoints(text);
  const points = paragraphs.length > 0 ? paragraphs : lines;

  if (points.length === 0) {
    return null;
  }

  const middle = text.length / 2;
  const cut = points.reduce((best, point) => (Math.abs(point - middle) < Math.abs(best - middle) ? point : best));

  return { halves: [text.slice(0, cut), text.slice(cut)], joint: paragraphs.length > 0 ? "\n\n" : "\n" };
}

// A text as its sections: the lines before the first heading, then each heading with the lines
// under it up to the next heading of any depth. Joined back, the sections are the text.
export function markdownSections(source: string): string[] {
  const lines = source.split("\n");
  const starts = markdownHeadings(source).map((heading) => heading.line - 1).filter((start) => start > 0);
  const sections: string[] = [];
  let from = 0;

  for (const start of starts) {
    sections.push(`${lines.slice(from, start).join("\n")}\n`);
    from = start;
  }

  sections.push(lines.slice(from).join("\n"));

  return sections;
}
