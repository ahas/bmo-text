import { describe, expect, test } from "bun:test";

import { withoutDashes } from "../dashes";

describe("a dash left between words", () => {
  test("a name and its explanation are joined by a colon", () => {
    expect(withoutDashes("Drone survey method — high resolution grid")).toBe("Drone survey method: high resolution grid");
    expect(withoutDashes("- **Owner** — the account holder")).toBe("- **Owner**: the account holder");
    expect(withoutDashes("## Why these rules — a real case")).toBe("## Why these rules: a real case");
  });

  test("an aside between two dashes is set off by commas, or by brackets where it has commas of its own", () => {
    expect(withoutDashes("I think — to be honest — that is right.")).toBe("I think, to be honest, that is right.");
    expect(withoutDashes("Anything larger — a feature, a refactor, a rulebook — is filed as an item.")).toBe(
      "Anything larger (a feature, a refactor, a rulebook) is filed as an item.",
    );
    expect(withoutDashes("It keeps the words — all of them —, then stops.")).toBe("It keeps the words, all of them, then stops.");
  });

  test("past an aside, a dash in the same sentence is a colon or a comma, and a new sentence starts over", () => {
    const text = "Lockdown mode — blocked when on — root is stopped too — no exceptions. Next item — TPM";

    expect(withoutDashes(text)).toBe("Lockdown mode, blocked when on, root is stopped too: no exceptions. Next item: TPM");
    expect(withoutDashes("MoniKa — recovered from memory, MIT.[5] Kakaotalk_decDB — an old-style example.")).toBe(
      "MoniKa: recovered from memory, MIT.[5] Kakaotalk_decDB: an old-style example.",
    );
  });

  test("a sentence that already has a colon takes a comma, where a line's opening label is no such colon", () => {
    expect(withoutDashes("- Question: relay — an agent manager")).toBe("- Question: relay: an agent manager");
    expect(withoutDashes("Note: the conclusion is this: one — two")).toBe("Note: the conclusion is this: one, two");
    expect(withoutDashes("Now(20:57) the places passed are 3 — Sunrise")).toBe("Now(20:57) the places passed are 3: Sunrise");
  });

  test("dashes each inside their own brackets are no pair", () => {
    expect(withoutDashes("platforms(Alpha — sales 8T), SI(Beta — 13.9T) in order")).toBe("platforms(Alpha: sales 8T), SI(Beta, 13.9T) in order");
  });

  test("an unspaced dash between digits is a range, and one after a pause mark or an opening bracket goes", () => {
    expect(withoutDashes("3—5 items")).toBe("3~5 items");
    expect(withoutDashes("Done. — the next one")).toBe("Done. the next one");
    expect(withoutDashes("(— note)")).toBe("(note)");
  });

  test("a spaced en dash is a dash, and an unspaced one is a range that stays", () => {
    expect(withoutDashes("Title – subtitle, 2020–2024")).toBe("Title: subtitle, 2020–2024");
  });

  test("what is not prose keeps its dashes", () => {
    const text = [
      "| price | — |",
      "— a line head",
      "a line end —",
      "- — after a list mark",
      "the dash(—) letter, a separator \" — \"",
      "> a quote — somebody else's words",
      "`a — b` code",
      "quoted \"OAuth token refresh failed — run /login\" as it is, “Lovable — Powering” too",
      "https://example.com/a—b an address",
      "```",
      "x — y",
      "```",
    ].join("\n");

    expect(withoutDashes(text)).toBe(text);
  });

  test("a wrapped document reads a paragraph or a list item as one piece and keeps its line breaks", () => {
    const text = [
      "- Anything larger inside a repository —",
      "  a feature, a refactor — is filed as an item. Other work — the",
      "  host's own — is done by hand.",
      "- A new item — its own piece.",
    ].join("\n");
    const expected = [
      "- Anything larger inside a repository",
      "  (a feature, a refactor) is filed as an item. Other work, the",
      "  host's own, is done by hand.",
      "- A new item: its own piece.",
    ].join("\n");

    expect(withoutDashes(text, { wrapped: true })).toBe(expected);
    expect(withoutDashes("Every other piece\n— this one included — stays.", { wrapped: true })).toBe(
      "Every other piece,\nthis one included, stays.",
    );
    expect(withoutDashes("- **Long term — years.** What decides returns — fees, tax — is long.", { wrapped: true })).toBe(
      "- **Long term: years.** What decides returns (fees, tax) is long.",
    );
    expect(withoutDashes("line one —\nthe next line — end")).toBe("line one —\nthe next line: end");
  });

  test("text with no dash comes back as it was, and a second pass changes nothing", () => {
    const plain = "A text with no dash. As it was.";
    const once = withoutDashes("A — B — C. D — E — F — G");

    expect(withoutDashes(plain)).toBe(plain);
    expect(withoutDashes(once)).toBe(once);
  });
});
