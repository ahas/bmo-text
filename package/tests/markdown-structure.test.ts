import { describe, expect, test } from "bun:test";

import { headingSlug, markdownHeadings, markdownSections, plainHeading } from "../markdown-structure";

describe("the headings of a text", () => {
  test("are read in order with their depth, words and source line", () => {
    const headings = markdownHeadings("# Document\n\nBody\n\n## First part ##\n### Deeper `code`\n");

    expect(headings).toEqual([
      { depth: 1, text: "Document", slug: "document", line: 1 },
      { depth: 2, text: "First part", slug: "first-part", line: 5 },
      { depth: 3, text: "Deeper code", slug: "deeper-code", line: 6 },
    ]);
  });

  test("skip every line of a fenced code block, whichever fence opened it", () => {
    const source = ["# One", "```sh", "# comment", "~~~", "# still code", "```", "~~~~", "## not", "~~~~", "## Two"].join("\n");

    expect(markdownHeadings(source).map((heading) => heading.text)).toEqual(["One", "Two"]);
  });

  test("give a repeated heading a mark of its own", () => {
    const slugs = markdownHeadings("## Example\n## Example\n## Example-1\n## Example").map((heading) => heading.slug);

    expect(slugs).toEqual(["example", "example-1", "example-1-1", "example-2"]);
  });

  test("are not a hash without words, a hash without a space, or an indented block", () => {
    expect(markdownHeadings("#\n#nospace\n    # indented\n#### Four")).toEqual([{ depth: 4, text: "Four", slug: "four", line: 4 }]);
  });

  test("read a text written with carriage returns on the same lines", () => {
    expect(markdownHeadings("# a\r\n\r\n## b").map((heading) => heading.line)).toEqual([1, 3]);
  });
});

describe("a heading's words", () => {
  test("lose their markup", () => {
    expect(plainHeading("**bold** [link](https://example.org) `code` ~~struck~~ _slanted_")).toBe("bold link code struck slanted");
    expect(plainHeading("snake_case_name")).toBe("snake_case_name");
  });

  test("make a mark of letters and digits in any script", () => {
    expect(headingSlug("Gate runs (runGateModel)!")).toBe("gate-runs-rungatemodel");
    expect(headingSlug("휴식 상태 — 두 가지")).toBe("휴식-상태-두-가지");
    expect(headingSlug("!!!")).toBe("section");
  });
});

describe("a text's sections", () => {
  test("start at each heading, keep the lead before the first, and join back into the text", () => {
    const source = "Lead\n\n# One\nBody\n```\n# code\n```\n## Two\nEnd";
    const sections = markdownSections(source);

    expect(sections).toEqual(["Lead\n\n", "# One\nBody\n```\n# code\n```\n", "## Two\nEnd"]);
    expect(sections.join("")).toBe(source);
    expect(markdownSections("# From the start\nText")).toEqual(["# From the start\nText"]);
  });
});
