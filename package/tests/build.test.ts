import { readFileSync } from "node:fs";

import { expect, test } from "bun:test";

import { sourceHash, sourceHashFile } from "../build";

// The package answers through the build it carries, so the build must be made from the crate as it stands
// (docs/design.md, 「One implementation」).
test("the WebAssembly build is made from the crate's sources as they stand", () => {
  expect(readFileSync(sourceHashFile, "utf8").trim(), "the crate changed since the build: run `bun run build` in package/").toBe(
    sourceHash(),
  );
});
