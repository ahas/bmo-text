import { describe, expect, test } from "bun:test";

import { capItems, capValue, maxAnswerChars, maxValueChars } from "../answer-cap";
import cases from "./answer-cap-cases.json";

// The ceiling every verb answers under. What it has to do is cut to whole items, never inside one,
// and say what it left out — a short answer read as the whole answer is the failure this exists to
// prevent, so a run that cut something is never quiet about it.

const chars = (text: string) => [...text].length;

// A render as thin as a real one: one item per line, so a prefix's length is its item count.
const lines = (kept: readonly string[]) => kept.join("\n");
const said = { unit: "nodes", how: "ask with narrower words." };

describe("cutting a list to the ceiling", () => {
  test("a list that fits comes back whole, with nothing to say about it", () => {
    const capped = capItems(["one", "two", "three"], lines, said);

    expect(capped.kept).toEqual(["one", "two", "three"]);
    expect(capped.left).toBe(0);
    expect(capped.note).toBeNull();
  });

  test("an empty list is a list that fits", () => {
    expect(capItems([], lines, said)).toEqual({ kept: [], left: 0, note: null });
  });

  test("a list over the ceiling is cut to whole items, and the rest is counted", () => {
    const items = Array.from({ length: 40 }, () => "가".repeat(9));
    // Ten characters an item, nine plus the newline, so four hundred against a ceiling of a hundred.
    const capped = capItems(items, lines, { ...said, limit: 100 });

    expect(capped.kept).toHaveLength(10);
    expect(capped.left).toBe(30);
    expect(chars(lines(capped.kept))).toBeLessThanOrEqual(100);
  });

  test("and it is the largest run of items that fits, not merely some run that does", () => {
    const items = Array.from({ length: 40 }, () => "가".repeat(9));
    const capped = capItems(items, lines, { ...said, limit: 100 });

    // One more item would be over it, which is what makes this the boundary.
    expect(chars(lines(items.slice(0, capped.kept.length + 1)))).toBeGreaterThan(100);
  });

  test("the line says how many there were, how many came back, how many did not, and what to do", () => {
    const capped = capItems(Array.from({ length: 40 }, () => "가".repeat(9)), lines, { ...said, limit: 100 });

    expect(capped.note).toBe("-- cut at the 100-character ceiling: 10 of 40 nodes. For the other 30: ask with narrower words.");
  });

  test("counts of a thousand and over are grouped, the way every other count here is", () => {
    const capped = capItems(Array.from({ length: 4_000 }, () => "가".repeat(9)), lines, { ...said, limit: 100 });

    expect(capped.note).toContain("10 of 4,000 nodes");
    expect(capped.note).toContain("the other 3,990");
  });

  test("where one item alone is over the ceiling, nothing comes back and the line says that", () => {
    const capped = capItems(["가".repeat(500), "short"], lines, { ...said, limit: 100 });

    expect(capped.kept).toEqual([]);
    expect(capped.left).toBe(2);
    expect(capped.note).toBe("-- one of the nodes alone is over the 100-character ceiling, so nothing was given. ask with narrower words.");
  });

  test("the ceiling is measured against what would be printed, separators and all", () => {
    const items = ["one", "two", "three"];
    // Rendered as JSON with indentation, these three weigh far more than their own characters, and
    // it is the printed weight that the ceiling holds.
    const capped = capItems(items, (kept) => JSON.stringify(kept, null, 2), { ...said, limit: 20 });

    expect(capped.kept.length).toBeLessThan(items.length);
    expect(chars(JSON.stringify(capped.kept, null, 2))).toBeLessThanOrEqual(20);
  });

  test("characters are code points, so a Korean answer is measured as a reader counts it", () => {
    // Nine Hangul syllables are nine characters and twenty-seven UTF-8 bytes; a ceiling of ten
    // takes one such item, and would take none if bytes were being counted.
    expect(capItems(["가".repeat(9), "가".repeat(9)], lines, { ...said, limit: 10 }).kept).toHaveLength(1);
  });

  test("the standing ceiling is above what any ordinary answer weighs", () => {
    // The largest shell result an agent session was measured returning is 29,261 characters, so
    // nothing in ordinary use meets this.
    expect(maxAnswerChars).toBeGreaterThan(29_261);
  });
});

// The cases a Rust twin of the ceiling is held to as well, so the two answer a cut in the same words.
type Piece = string | { text: string; times: number };

const joined = (pieces: readonly Piece[]): string => pieces.map((piece) => (typeof piece === "string" ? piece : piece.text.repeat(piece.times))).join("");
const renders = { lines, json: (kept: readonly string[]) => JSON.stringify(kept, null, 2) };

describe("the cases shared with the Rust ceiling", () => {
  for (const one of cases.items as { name: string; items: Piece[]; copies?: number; limit?: number; render: "lines" | "json"; unit: string; how: string; kept: number; left: number; note: string | null }[]) {
    test(one.name, () => {
      const pieces = one.items.map((piece) => joined([piece]));
      const items = Array.from({ length: one.copies ?? 1 }, () => pieces).flat();
      const capped = capItems(items, renders[one.render], { unit: one.unit, how: one.how, ...(one.limit === undefined ? {} : { limit: one.limit }) });

      expect([capped.kept.length, capped.left, capped.note]).toEqual([one.kept, one.left, one.note]);
    });
  }

  for (const one of cases.values as { name: string; text: Piece[]; limit?: number; expect: Piece[] }[]) {
    test(one.name, () => {
      expect(capValue(joined(one.text), one.limit)).toBe(joined(one.expect));
    });
  }
});

describe("cutting one long value", () => {
  test("a value that fits is handed back exactly as it stands", () => {
    expect(capValue("a short value")).toBe("a short value");
    expect(capValue("가".repeat(maxValueChars))).toHaveLength(maxValueChars);
  });

  test("a long one is cut where it stands, with how much was cut said in place of the rest", () => {
    expect(capValue("가".repeat(120), 100)).toBe(`${"가".repeat(100)}…(+20 chars)`);
  });

  test("and what was cut is counted in code points, not in what it takes to store them", () => {
    expect(capValue(`${"가".repeat(100)}${"🙂".repeat(20)}`, 100)).toBe(`${"가".repeat(100)}…(+20 chars)`);
  });

  test("a surrogate pair is never torn in half by the cut", () => {
    const cut = capValue("🙂".repeat(20), 10);

    expect([...cut.slice(0, cut.indexOf("…"))]).toHaveLength(10);
    expect(cut).toContain("…(+10 chars)");
  });
});
