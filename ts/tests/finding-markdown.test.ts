import { describe, expect, test } from "bun:test";

import { englishFindingWords, findingExcerpt, findingMarkdown, pictureCaption } from "../finding-markdown";

// The words a link's preview card describes a document by: its opening prose, plain.
describe("a document's opening words", () => {
  test("are the key answer's and the summary's words, without list marks, footnote marks or inline markup", () => {
    const summary = ["VERDICT: Corner Kitchen[2].", "SUMMARY:", "1. The cheapest place is **Corner Kitchen**[2].", "2. [Blue Door](https://example.com) is `8,000 won`[3†]."].join("\n");

    expect(findingExcerpt([summary], 200)).toBe("Corner Kitchen. The cheapest place is Corner Kitchen. Blue Door is 8,000 won.");
  });

  test("leave out what is drawn rather than said, and what is folded under MORE", () => {
    const entry = [
      "NAME: Corner Kitchen",
      "# Heading",
      "IMAGE: https://example.com/p.jpg :: photo",
      "| menu | price |",
      "---",
      "ADDRESS: Riverside",
      "MENU:",
      "- noodles :: 6,000 won",
      "SUMMARY: A noodle house.",
      "> It has a self-serve bar.",
      "MORE:",
      "a folded story"
    ].join("\n");

    expect(findingExcerpt([entry, "next finding"], 200)).toBe("A noodle house. It has a self-serve bar. next finding");
  });

  test("are cut at the last space before the limit and closed with an ellipsis", () => {
    expect(findingExcerpt(["aa b c d ee", "ff"], 12)).toBe("aa b c d…");
    expect(findingExcerpt(["NAME: only a name"], 12)).toBe("");
  });
});

// The words a drawn finding shows are its consumer's; with none passed, a part is called by its key
// and a line by its English label.
describe("a finding's drawn words", () => {
  test("are English by default, a part called by its key", () => {
    expect(findingMarkdown("VERDICT: yes")).toBe("\n**VERDICT** yes");
    expect(findingMarkdown("ADDRESS: Riverside")).toBe("**Address** Riverside");
  });

  test("are the consumer's where it passes its own", () => {
    const words = { ...englishFindingWords, parts: { VERDICT: "Answer" }, address: "Where" };

    expect(findingMarkdown("VERDICT: yes", words)).toBe("\n**Answer** yes");
    expect(findingMarkdown("ADDRESS: Riverside", words)).toBe("**Where** Riverside");
  });
});

// A picture's caption where it is shown: what it shows, then its shot and whose picture it is.
describe("a picture's shown caption", () => {
  test("is its caption with the shot and the source after it, the entry left to the heading above", () => {
    expect(pictureCaption("garden group :: Studio A :: outdoor :: official")).toBe("garden group · outdoor · official");
    expect(pictureCaption("soup 9,000")).toBe("soup 9,000");
  });
});
