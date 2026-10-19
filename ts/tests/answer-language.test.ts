import { describe, expect, test } from "bun:test";

import { holdsKorean } from "../answer-language";

describe("whether a piece of text holds Korean", () => {
  test("a Korean sentence does", () => {
    expect(holdsKorean("#524 Job을 띄웠어요.")).toBe(true);
  });

  test("an English sentence does not", () => {
    expect(holdsKorean("I started the job for #524.")).toBe(false);
  });

  test("a code span or a path alone does not", () => {
    expect(holdsKorean("`bunx tsc --noEmit`")).toBe(false);
  });
});

