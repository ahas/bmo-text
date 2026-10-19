import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, test } from "bun:test";

import {
  addBannedTerms,
  type BannedTerm,
  bannedTermsIn,
  bannedTermsLine,
  checkWording,
  findBanned,
  lineLanguage,
  parseGlossary,
  readGlossary,
  wordingDocument,
} from "../wording";
import cases from "../../cases/wording.json";
import { holdAll } from "./shared";

// A refusal is compared as the sentence it gives, and the terms a text uses are found against the banned
// list alone.
function refused<T>(read: () => T): T | { error: string } {
  try {
    return read();
  } catch (error) {
    return { error: (error as Error).message };
  }
}

holdAll("wording", cases, {
  lineLanguage,
  parseGlossary: (text: string) => refused(() => parseGlossary(text)),
  findBanned,
  bannedTermsIn: (text: string, banned: BannedTerm[]) => bannedTermsIn(text, { banned, allowed: [] }),
  bannedTermsLine,
  addBannedTerms: (text: string, entries: BannedTerm[]) => refused(() => addBannedTerms(text, entries)),
});

const sample = [
  "## Banned terms",
  "",
  "- en · tool strip → toolbar",
  "- en · fixing → editing — when it means editing",
  "",
  "## Allowed uses",
  "",
  "- en · fixing · docs/a.md · bike fixing — a repair",
].join("\n");

// What the package carries and the crate does not: a glossary read from a folder, and files held to it.
describe("the package's own reading of a glossary", () => {
  test("the glossary is read from the folder the caller names, under its conventional name", () => {
    const root = mkdtempSync(join(tmpdir(), "markup-wording-"));

    try {
      writeFileSync(join(root, wordingDocument), sample);

      expect(readGlossary(root)).toEqual(parseGlossary(sample));
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
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
