// An HTML file read for what it says, so a document can be written out of it. A page handed over
// as a file is often mostly pictures written into it as data addresses — megabytes a reader could
// never open whole and would learn nothing from — so the reading keeps the words in their order and
// takes every such picture out as a file of its own, named where it stood.
//
// The reading is structural and nothing more: headings keep their level as markdown headings, a
// list item is a dash, every block starts a line, and scripts and styles are not words. It does not
// run the page, so what a script would draw is not in it.

import { decodeHTML } from "entities";

export interface HtmlPicture {
  // The name it stands under in the reading, and the one it is kept by.
  name: string;
  type: string;
  bytes: Uint8Array;
}

export interface HtmlReading {
  title: string | null;
  // What the page says of itself in its `description` meta, where it says anything.
  description: string | null;
  // The page's words, a block to a line, with pictures as markdown pictures where they stood.
  lines: string[];
  // Every picture the page carried inside itself, in the order they stood.
  pictures: HtmlPicture[];
}

const BLOCKS = new Set([
  "address", "article", "aside", "blockquote", "br", "button", "dd", "details", "div", "dl", "dt", "figcaption", "figure",
  "footer", "form", "header", "hr", "li", "main", "nav", "ol", "p", "pre", "section", "small", "summary", "table", "td",
  "th", "tr", "ul"
]);
const UNREAD = new Set(["script", "style", "noscript", "template", "svg", "head"]);
const HEADING = /^h([1-6])$/;
const DESCRIPTION_META = /^(?:description|og:description)$/i;
const DATA_PICTURE = /^data:(image\/[\w.+-]+)(;base64)?,(.*)$/s;

const EXTENSIONS: Record<string, string> = {
  "image/jpeg": ".jpg",
  "image/png": ".png",
  "image/gif": ".gif",
  "image/webp": ".webp",
  "image/avif": ".avif",
  "image/svg+xml": ".svg"
};

// A payload's escapes read as a URI component's, or the payload as it stands where they are not well formed.
function unescaped(payload: string): string {
  try {
    return decodeURIComponent(payload);
  } catch {
    return payload;
  }
}

function pictureBytes(match: RegExpExecArray): Uint8Array {
  const payload = match[3] ?? "";

  return match[2] === undefined ? new TextEncoder().encode(unescaped(payload)) : new Uint8Array(Buffer.from(payload, "base64"));
}

// The reading of one page. `stem` is what the pictures taken out of it are named after, numbered
// in the order they stood.
export function readHtml(html: string, stem = "picture"): HtmlReading {
  const lines: string[] = [];
  const pictures: HtmlPicture[] = [];
  let line = "";
  let prefix = "";
  let unread = 0;
  let title: string | null = null;
  let description: string | null = null;
  let inTitle = false;

  const flush = (): void => {
    const words = decodeHTML(line).replace(/\s+/g, " ").trim();

    line = "";

    if (words !== "") {
      lines.push(`${prefix}${words}`);
      prefix = "";
    }
  };

  const picture = (source: string, alt: string): string => {
    const data = DATA_PICTURE.exec(source);

    if (data === null) return `![${alt}](${source})`;

    const type = data[1]!.toLowerCase();
    const name = `${stem}-${String(pictures.length + 1).padStart(2, "0")}${EXTENSIONS[type] ?? `.${type.slice(6).split("+")[0]}`}`;

    pictures.push({ name, type, bytes: pictureBytes(data) });

    return `![${alt}](${name})`;
  };

  new HTMLRewriter()
    .on("*", {
      element(element) {
        const tag = element.tagName.toLowerCase();

        if (tag === "meta" && description === null && DESCRIPTION_META.test(element.getAttribute("name") ?? element.getAttribute("property") ?? "")) {
          const content = decodeHTML(element.getAttribute("content") ?? "").replace(/\s+/g, " ").trim();

          if (content !== "") description = content;

          return;
        }

        // An element that cannot hold content, such as a self-closing one in SVG, has no end tag to wait for.
        if (tag === "title") {
          if (!element.canHaveContent) return;

          inTitle = true;
          element.onEndTag(() => {
            inTitle = false;
          });

          return;
        }

        if (UNREAD.has(tag)) {
          if (!element.canHaveContent) return;

          unread += 1;
          element.onEndTag(() => {
            unread -= 1;
          });

          return;
        }

        if (unread > 0) return;

        // Words in two neighbouring elements are two words, however the page set them side by side.
        line += " ";

        const heading = HEADING.exec(tag);

        if (tag === "img") {
          flush();
          lines.push(picture(element.getAttribute("src") ?? "", decodeHTML(element.getAttribute("alt") ?? "")));

          return;
        }

        if (heading !== null || tag === "li" || BLOCKS.has(tag)) {
          flush();
          // A block inside a heading or a list item keeps the mark it stands under.
          if (heading !== null) prefix = `${"#".repeat(Number(heading[1]))} `;
          if (tag === "li") prefix = "- ";

          if (element.canHaveContent && !element.selfClosing && tag !== "br" && tag !== "hr") {
            element.onEndTag(() => {
              if (unread === 0) flush();
            });
          }
        }
      }
    })
    .onDocument({
      text(chunk) {
        if (inTitle) {
          title = `${title ?? ""}${chunk.text}`;
        } else if (unread === 0) {
          line += chunk.text;
        }
      }
    })
    .transform(html);

  flush();

  const cleanTitle = title === null ? null : decodeHTML(title).replace(/\s+/g, " ").trim();

  return { title: cleanTitle === "" ? null : cleanTitle, description, lines, pictures };
}

