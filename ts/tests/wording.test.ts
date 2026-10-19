import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, test } from "bun:test";

import { addBannedTerms, bannedTermsIn, bannedTermsLine, checkWording, findBanned, lineLanguage, parseGlossary, readGlossary, wordingDocument } from "../wording";

const sample = [
  "# Wording",
  "",
  "## Banned terms",
  "",
  "- Each line is `<language code> · <banned term> → <what to write>`.",
  "- en · tool strip → toolbar",
  "- en · fixing → editing — when it means editing",
  "- en · slot → position",
  "",
  "## Uses in the word's own sense",
  "",
  "- Each line is `<language code> · <banned term> · <a phrase holding the term>`.",
  "- en · slot · slot machine — a machine",
  "- en · slot · coin slot — an opening",
  "",
  "## Allowed uses",
  "",
  "- en · fixing · docs/a.md · bike fixing — a repair",
].join("\n");

describe("reading the glossary", () => {
  test("entries are read from their own sections and the explanation bullets are not", () => {
    const glossary = parseGlossary(sample);

    expect(glossary.banned).toEqual([
      { lang: "en", term: "tool strip", instead: "toolbar" },
      { lang: "en", term: "fixing", instead: "editing", when: "when it means editing" },
      { lang: "en", term: "slot", instead: "position", keeps: ["slot machine", "coin slot"] },
    ]);
    expect(glossary.allowed).toEqual([{ lang: "en", term: "fixing", path: "docs/a.md", excerpt: "bike fixing" }]);
  });

  test("new banned lines go after the last entry of the list and read back as written", () => {
    const { text, added } = addBannedTerms(sample, [
      { lang: "en", term: "stashing", instead: "saving, capturing", when: "when it means saving" },
      { lang: "en", term: "slot", instead: "position" },
    ]);
    const lines = text.split("\n");

    expect(added.map((one) => one.term)).toEqual(["stashing"]);
    expect(lines[lines.indexOf("- en · slot → position") + 1]).toBe("- en · stashing → saving, capturing — when it means saving");
    expect(parseGlossary(text).banned.at(-1)).toEqual({ lang: "en", term: "stashing", instead: "saving, capturing", when: "when it means saving" });
    expect(addBannedTerms(sample, [{ lang: "en", term: "slot", instead: "position" }]).text).toBe(sample);
  });

  test("a banned line that would read back otherwise is refused", () => {
    expect(() => addBannedTerms(sample, [{ lang: "en", term: "a → b", instead: "c" }])).toThrow(/would not read back/);
  });

  test("a language code the check cannot tell is refused", () => {
    expect(() => parseGlossary("## Banned terms\n- kr · built in → built-in")).toThrow(/unknown language code "kr"/);
  });

  test("a phrase of a word's own sense that does not hold a banned term is refused", () => {
    const own = "\n## Uses in the word's own sense\n";

    expect(() => parseGlossary(`## Banned terms\n- en · slot → position${own}- en · slot · seat booking — a seat`)).toThrow(/must hold a banned term/);
    expect(() => parseGlossary(`## Banned terms\n- en · slot → position${own}- en · seat · seat booking — a seat`)).toThrow(/must hold a banned term/);
  });

  test("the glossary is read from the folder the caller names, under its conventional name", () => {
    const root = mkdtempSync(join(tmpdir(), "markup-wording-"));

    try {
      writeFileSync(join(root, wordingDocument), sample);

      expect(readGlossary(root)).toEqual(parseGlossary(sample));
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

describe("finding a banned term", () => {
  test("a line is Korean when it holds Hangul, Japanese when it holds kana alone, English otherwise", () => {
    expect(lineLanguage("툴바를 눌러")).toBe("ko");
    expect(lineLanguage("ツールバー")).toBe("ja");
    expect(lineLanguage("press the toolbar")).toBe("en");
  });

  test("a term is found only on lines of its own language", () => {
    const en = [{ lang: "en", term: "tool strip", instead: "toolbar" }];
    const text = "위에 tool strip 있음\nthe tool strip above";

    expect(findBanned("x.md", text, en).map((hit) => hit.line)).toEqual([2]);
  });

  test("a Korean term is not found where a syllable runs into it from the left", () => {
    const ko = [{ lang: "ko", term: "돌리기", instead: "실행" }];

    expect(findBanned("x.md", "협업으로 돌리기", ko)).toHaveLength(1);
    expect(findBanned("x.md", "결재를 되돌리기", ko)).toHaveLength(0);
    // The word is still the term's own where a particle or an ending follows it, and a line that
    // carries both forms is reported for the one that stands on its own.
    expect(findBanned("x.md", "돌리기도 한다", ko)).toHaveLength(1);
    expect(findBanned("x.md", "되돌리기와 다시 돌리기", ko)).toHaveLength(1);
  });

  test("a term inside a phrase of its own sense passes, and the same term outside one is still found", () => {
    const banned = parseGlossary(sample).banned;

    expect(findBanned("x.md", "the slot machine paid out", banned)).toHaveLength(0);
    expect(findBanned("x.md", "drop it in the coin slot", banned)).toHaveLength(0);
    expect(findBanned("x.md", "the slot where it stopped", banned)).toHaveLength(1);
    expect(findBanned("x.md", "a coin slot and the slot where it stopped", banned)).toHaveLength(1);
  });

  test("an allowed use passes only in its own file and line, and a stale one is reported", () => {
    const glossary = parseGlossary(sample);
    const files: Record<string, string> = {
      "docs/a.md": "bike fixing\ndocument fixing\n",
      "docs/b.md": "bike fixing\nclick the tool strip\n",
    };
    const report = checkWording(Object.keys(files), (path) => files[path]!, glossary);

    expect(report.hits.map((hit) => `${hit.path}:${hit.line}:${hit.term.term}`)).toEqual([
      "docs/a.md:2:fixing",
      "docs/b.md:1:fixing",
      "docs/b.md:2:tool strip",
    ]);
    expect(report.unusedAllowances).toEqual([]);
    expect(checkWording([], () => "", glossary).unusedAllowances).toHaveLength(1);
  });
});

// A text being written — a row's title, an answer, a report — is held to the same list, less the
// spans it quotes, which is how a text names a wrong form on purpose.
describe("holding a text being written", () => {
  const glossary = parseGlossary(sample);

  test("a term is named once, beside what to write, however often it is used", () => {
    const terms = bannedTermsIn("click the tool strip, then the tool strip again", glossary);

    expect(terms.map((one) => one.term)).toEqual(["tool strip"]);
    expect(bannedTermsLine(terms)).toBe('"tool strip" → toolbar');
  });

  test("a term inside backticks or quotation marks passes", () => {
    for (const text of ['toolbar, not "tool strip"', "`tool strip` is the old name", "「tool strip」 is not what it is called", "'fixing' is not editing"]) {
      expect({ text, terms: bannedTermsIn(text, glossary) }).toEqual({ text, terms: [] });
    }
  });

  test("the word-head rule holds in a text being written", () => {
    const korean = parseGlossary("## Banned terms\n- ko · 고치기 → 수정");

    expect(bannedTermsIn("설계를 뜯어고치기", korean)).toEqual([]);
    expect(bannedTermsIn("문서 고치기", korean).map((one) => one.term)).toEqual(["고치기"]);
  });
});
