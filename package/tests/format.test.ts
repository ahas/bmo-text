import { describe, expect, test } from "bun:test";

import { escapeHtml, expandable, markupFault, visibleText } from "../format";

const tricky = `a < b & c > d`;
const markupish = `<b>bold</b> & <blockquote expandable>expanded</blockquote>`;

describe("escapeHtml", () => {
  test("escapes the three characters HTML mode reads", () => {
    expect(escapeHtml(tricky)).toBe("a &lt; b &amp; c &gt; d");
  });

  test("escapes the ampersand first, so an escape is never double-escaped wrongly", () => {
    expect(escapeHtml("&lt;")).toBe("&amp;lt;");
  });

  test("leaves text that looks like markup as literal characters", () => {
    expect(escapeHtml(markupish)).toBe(
      "&lt;b&gt;bold&lt;/b&gt; &amp; &lt;blockquote expandable&gt;expanded&lt;/blockquote&gt;",
    );
  });
});

describe("expandable", () => {
  test("escapes what it hides", () => {
    expect(expandable(tricky)).toBe("<blockquote expandable>a &lt; b &amp; c &gt; d</blockquote>");
  });
});

describe("visibleText, the rung the failed gate stands on", () => {
  test("what a reader is left with is the words, without the tags or the escapes", () => {
    expect(visibleText("<b>bold</b> and <code>code</code>")).toBe("bold and code");
    expect(visibleText("<blockquote expandable>folded words</blockquote>")).toBe("folded words");
    expect(visibleText("&lt;words that look like a tag&gt; &amp; after them")).toBe("<words that look like a tag> & after them");
  });

  test("markup with no word left in it reads as nothing, which is what the ladder steps over", () => {
    expect(visibleText("<b></b>").trim()).toBe("");
    expect(visibleText("<blockquote expandable></blockquote>").trim()).toBe("");
    expect(visibleText("").trim()).toBe("");
  });
});

describe("markupFault, the rulebook's tags read by the code", () => {
  test("the markup this module makes has no fault in it", () => {
    expect(markupFault("<b>title</b>")).toBeNull();
    expect(markupFault(`<code>${escapeHtml(tricky)}</code>`)).toBeNull();
    expect(markupFault('<a href="https://example.com/?a=1&amp;b=&quot;2&quot;">here</a>')).toBeNull();
    expect(markupFault(expandable("folded words"))).toBeNull();
    expect(markupFault("<blockquote expandable><b>title</b>\n<code>value</code></blockquote>")).toBeNull();
    // The one nesting a literal is allowed: a code block naming its language.
    expect(markupFault('<pre><code class="language-python">print(1)</code></pre>')).toBeNull();
    expect(markupFault('<span class="tg-spoiler">ending</span>')).toBeNull();
  });

  test("catches code inside a formatting tag, and formatting inside code", () => {
    expect(markupFault("<b>title <code>value</code></b>")).toEqual({ kind: "nested", tag: "code", inside: "b" });
    expect(markupFault("<code>value <b>bold</b></code>")).toEqual({ kind: "nested", tag: "b", inside: "code" });
    expect(markupFault('<a href="https://example.com/">here <code>value</code></a>')).toEqual({
      kind: "nested",
      tag: "code",
      inside: "a",
    });
  });

  test("catches a tag that never closes, and one that closes out of turn", () => {
    expect(markupFault("<b>title")).toEqual({ kind: "unclosed", tag: "b" });
    expect(markupFault("<blockquote expandable>body")).toEqual({ kind: "unclosed", tag: "blockquote" });
    expect(markupFault("<b><i>title</b></i>")).toEqual({ kind: "unopened", tag: "b", open: "i" });
    expect(markupFault("title</b>")).toEqual({ kind: "unopened", tag: "b", open: null });
  });

  test("catches a tag Telegram does not know, including a span that is not a spoiler", () => {
    expect(markupFault("<div>title</div>")).toEqual({ kind: "unsupported", tag: "div" });
    expect(markupFault("one line<br>the next line")).toEqual({ kind: "unsupported", tag: "br" });
    expect(markupFault("<span>title</span>")).toEqual({ kind: "unsupported", tag: "span" });
    // Supported by Telegram, not used by a chat service here, so not a tag the gate may send.
    expect(markupFault('<tg-emoji emoji-id="5368324170671202286">🙂</tg-emoji>')).toEqual({
      kind: "unsupported",
      tag: "tg-emoji",
    });
  });

  test("catches a < > or & the escaping missed outside a tag", () => {
    expect(markupFault("a < b")).toEqual({ kind: "unescaped", character: "<" });
    expect(markupFault("a > b")).toEqual({ kind: "unescaped", character: ">" });
    expect(markupFault("R&D")).toEqual({ kind: "unescaped", character: "&" });
    expect(markupFault("<b>R&D</b>")).toEqual({ kind: "unescaped", character: "&" });
    expect(markupFault("&lt;a&gt; &amp; &quot; &#39; &#x1F600;")).toBeNull();
  });

  test("catches an expandable quote inside a quote", () => {
    expect(markupFault("<blockquote>somebody's words <blockquote expandable>folded again</blockquote></blockquote>")).toEqual({
      kind: "nested",
      tag: "blockquote",
      inside: "blockquote",
    });
  });
});

// Candidates a verifier turned down for a structure violation (code nested in bold, code inside
// code) that the tags say is not there. Three of them are one candidate made three times over,
// byte for byte the same, and each time the reason named a different tag.
const resumed = `The alert was a false one. The temporary folders the tests make were read as real runs.

There are three reasons.
1. The tests use a fake runner and remove its records when they end.
2. The log line that raised the alert is the same as a test phrase.
3. The usage record shows no such burst at that time.

What was really spent in the same window was small, and no run of the last hour went cold.`;

describe("markupFault, the candidates the verifier called broken", () => {
  const called = [
    { case: "#261", markup: `An update, in short. The checks are done and the two faults that kept returning are closed.

What I counted
<code>17</code> of the failures came from one reader; the rest were spread across four stages.

What I fixed
One place now takes the markers off before any tag is put on, so a marker and a tag can no longer overlap.` },
    { case: "#275", markup: `A finished report can now be opened again with its earlier results kept. The earlier round's result documents stay, and the new plan reads them before it starts.

What I checked
<blockquote expandable>The command was run on a copy of the record, and the console's button was tried on that copy. Both opened the report again with the earlier round kept apart.</blockquote>

Nothing else changed.` },
    { case: "#293", markup: resumed },
    { case: "#294", markup: resumed },
    { case: "#295", markup: resumed },
  ];

  for (const one of called) {
    test(`${one.case} has no fault in its tags`, () => {
      expect(markupFault(one.markup)).toBeNull();
    });
  }
});
