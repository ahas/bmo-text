import { describe, expect, test } from "bun:test";

// The cases a module is held to on both sides, read from its file under `cases/` (docs/design.md, 「The
// twins」). Each function the file names is answered here, and every case of it is held.

interface Case {
  name: string;
  args: unknown[];
  answer: unknown;
}

// A case's arguments are whatever its function takes.
type Answer = (...args: any[]) => unknown;

export function holdAll(module: string, file: Record<string, unknown>, answers: Record<string, Answer>): void {
  describe(`the cases shared with the crate: ${module}`, () => {
    test("the functions the file names are the ones answered", () => {
      expect(Object.keys(file).filter((key) => key !== "about").sort()).toEqual(Object.keys(answers).sort());
    });

    for (const [name, answer] of Object.entries(answers)) {
      for (const one of (file[name] ?? []) as Case[]) {
        test(`${name}: ${one.name}`, () => {
          // An answer is compared as the JSON it is written in, so an absent field and a `null` read alike.
          expect(JSON.parse(JSON.stringify(answer(...one.args)) ?? "null")).toEqual(one.answer);
        });
      }
    }
  });
}
