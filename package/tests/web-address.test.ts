import { describe, expect, test } from "bun:test";

import { addressAsWritten, gluedAfterAddress, webAddresses } from "../web-address";

describe("the web addresses a text names", () => {
  test("a markdown link gives the address its parentheses hold, and nothing written after them", () => {
    const text = "결과는 [archive 83](https://site.example.com/archive/83)에 올려 뒀어.";

    const url = "https://site.example.com/archive/83";
    const start = text.indexOf(url);

    expect(webAddresses(text)).toEqual([{ start, end: start + url.length, url, label: "archive 83" }]);
  });

  test("a markdown link keeps a pair of brackets inside its address and may hold Korean letters", () => {
    expect(webAddresses("[위키](https://en.wikipedia.org/wiki/Foo_(bar))에 [서울](https://ko.wikipedia.org/wiki/서울)을").map(({ url }) => url)).toEqual([
      "https://en.wikipedia.org/wiki/Foo_(bar)",
      "https://ko.wikipedia.org/wiki/서울"
    ]);
  });

  test("a bare address ends at a Korean letter, at a closing bracket it did not open and before the sentence's punctuation", () => {
    expect(webAddresses("주소 https://a.example/83에 있고 (https://b.example/p_(q))에, https://c.example/x?y=1. 끝").map(({ url }) => url)).toEqual([
      "https://a.example/83",
      "https://b.example/p_(q)",
      "https://c.example/x?y=1"
    ]);
    expect(addressAsWritten("https://site.example.com/archive/83)에")).toBe("https://site.example.com/archive/83");
  });

  test("a scheme alone, a link to something other than the web and plain words name no address", () => {
    expect(webAddresses("https:// is no address, [text](ftp://x.example) is not the web, and these are plain words")).toEqual([]);
  });

  test("the sentence runs on after an address only where a Korean letter follows with no space", () => {
    const after = (written: string): boolean => gluedAfterAddress(written, "https://a.example/83".length);

    expect(after("https://a.example/83에")).toBe(true);
    expect(after("https://a.example/83)에")).toBe(true);
    expect(after("https://a.example/83 에")).toBe(false);
    expect(after("https://a.example/83) 끝")).toBe(false);
  });
});
