import { describe, expect, test } from "bun:test";

import { readHtml } from "../html-reading";

// What a page handed over as a file reads as: its words in order, its structure as markdown, and the
// pictures written inside it taken out as files named where they stood.

const dot = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

describe("an HTML page read for its words", () => {
  test("keeps headings, blocks and list items as markdown lines, and leaves scripts and styles out", () => {
    const reading = readHtml(
      "<!doctype html><title> Palette &amp; colour </title><style>b{color:red}</style><h1>Gallery</h1><p>One line<br>Two lines</p>" +
        "<ul><li><p>Item</p></li></ul><script>alert(1)</script><section><h2>Pearl &amp; Cyan</h2>" +
        "<button><b>Pearl White</b><em>#FEFEFE · Dominant</em></button></section>"
    );

    expect(reading.title).toBe("Palette & colour");
    expect(reading.lines).toEqual(["# Gallery", "One line", "Two lines", "- Item", "## Pearl & Cyan", "Pearl White #FEFEFE · Dominant"]);
    expect(reading.pictures).toEqual([]);
  });

  test("takes every picture written inside the page out as a file, and names an outside one by its address", () => {
    const reading = readHtml(`<img src="data:image/png;base64,${dot}" alt="dot"><p>Between</p><img src="https://example.com/a.webp"><img src="data:image/svg+xml,%3Csvg%2F%3E">`, "palette");

    expect(reading.lines).toEqual(["![dot](palette-01.png)", "Between", "![](https://example.com/a.webp)", "![](palette-02.svg)"]);
    expect(reading.pictures.map((picture) => [picture.name, picture.type])).toEqual([["palette-01.png", "image/png"], ["palette-02.svg", "image/svg+xml"]]);
    expect([...reading.pictures[0]!.bytes.subarray(1, 4)].map((byte) => String.fromCharCode(byte)).join("")).toBe("PNG");
    expect(new TextDecoder().decode(reading.pictures[1]!.bytes)).toBe("<svg/>");
  });
});

