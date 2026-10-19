import { expect, test } from "bun:test";

import { FINDING_BLOCK, FINDING_MAP, findingExcerpt, findingMarkdown, footnoteMarks, pictureCaption } from "../finding-markdown";
import cases from "../../cases/finding-markdown.json";
import { holdAll } from "./shared";

holdAll("finding-markdown", cases, { findingMarkdown, footnoteMarks, pictureCaption, findingExcerpt });

test("a drawn gallery and map are the blocks a renderer reads", () => {
  const drawn = findingMarkdown(
    "NAME: Corner Kitchen\nADDRESS: Riverside\nMAP: https://map.example.com/a\nIMAGE: https://example.com/a.jpg :: one\nIMAGE: https://example.com/b.jpg :: two"
  );

  expect(FINDING_BLOCK.exec(drawn.slice(drawn.indexOf("::: gallery")))?.[1]).toBe("gallery");
  expect(FINDING_MAP.exec(drawn.slice(drawn.indexOf("::: map")))?.[1]).toBe("Riverside");
});
