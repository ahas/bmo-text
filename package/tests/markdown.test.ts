import { describe, expect, test } from "bun:test";

import { markupFault, visibleText } from "../format";
import {
  codeMarkup,
  convertMarkdown,
  foldBlocks,
  foldedMarkup,
  markdownParts,
  parseInline,
  parseMarkdown,
  plainInline,
  renderInline,
} from "../markdown";
import cases from "../../cases/markdown.json";
import { holdAll } from "./shared";

// The markdown an agent writes becomes Telegram HTML here, and whatever the input, three things hold:
// the markup has only tags Telegram takes and passes the structure check, no marker is left standing
// beside the tag it made, and nothing Telegram has no tag for survives as its marker.

const formattingTag = "(?:b|i|s|tg-spoiler|code|pre|a|blockquote)";
const markerBesideTag = new RegExp(
  `(?:\\*\\*|~~|\\|\\||\`)<\\/?${formattingTag}\\b|<${formattingTag}(?:\\s[^>]*)?>(?:\\*\\*|~~|\\|\\||\`)|(?:\\*\\*|~~|\\|\\||\`)<\\/${formattingTag}>|<\\/${formattingTag}>(?:\\*\\*|~~|\\|\\||\`)`,
);
const blockMarker = /^[ \t]*(?:#{1,6}[ \t]|[-*+][ \t]|\|.*\||-{3,}[ \t]*$|```|~~~)/;

// Every line of the markup outside a code block, with its tags taken out.
function shownLines(markup: string): string[] {
  return visibleText(markup.replace(/<pre>[\s\S]*?<\/pre>/g, "")).split("\n");
}

function expectClean(markup: string): void {
  expect(markupFault(markup)).toBeNull();
  expect(markup).not.toMatch(markerBesideTag);
  expect(shownLines(markup).filter((line) => blockMarker.test(line))).toEqual([]);
}

// Answers of the shapes the output gate turned down: report sections of plain lines, stars that are
// no pair, ranges written with a tilde, angle brackets in a command, circled numbers, paths, and a
// code block among backticks.
const corpus = [
  { case: "plain report", text: `A summary of the checks that ran overnight is done. I grouped the failures by kind, found the causes, and closed the two that kept returning in code, deployed.

What I counted
The 17 cases of a pair beside a tag came from the reader taking the backticks off and leaving the star pairs. 1~3, refused three times this round, are those, and 16 of them came after the second run.
Of the 44 backtick cases, 39 were already closed by the change on the fifth. What was left is the flaw of one value wrapped in backticks several times becoming code in one place only and the code blocks of many lines.
Several bold runs, fold borders, quoted messages, nested tags and the checker's answer shape all came before the fifth, and none since.
The other 46 are the first stage picking the wrong piece or the checker misjudging: 5 refused for the number of bold runs, 13 with formatting on the wrong piece, and 28 others.
Checking the same attempt three times wasted 29 of 173 runs.

What I fixed
The star pairs and backtick pairs an answer writes are now taken off where they are written, before any tag is put on. Bold or code goes in their place, so a star and a tag can no longer overlap. Backticks wrapping the same value several times become code in each place. 2**3, server/**/*.ts, stars inside backticks and stars with no pair stay as written.
An attempt the same as one already refused is no longer sent to the checker; it ends with the earlier verdict kept as its reason in the record.
The output rulebook and the checker's instruction were fixed first, and the code was made to match.
The side writing the answers is not stopped on its own. The manual already says not to write markup and it came in anyway, and since formatting is made in one place only, taking it off there shuts every path.

What I checked
I ran the 20 star cases and 1~3 again on the deployed code: 0 stars or backticks left, 0 tag structure faults, and the words as written.
I added 13 regression tests. The typecheck passed, and so did the 183 format, gate and agent tests.
Of the whole suite's 228 files, 3 failed. I fixed the agent test this change broke. A web test added for the change made its own temporary folder, and I fixed that in a commit of its own.
The last one is the code graph check, which fails on files another job has not committed yet.
After the deployment the service came up healthy.

What I left
Turning a code block of many lines \`\`\` into <pre> is not done. It is a feature of its own, and it happened once.
A checker misjudging a rule the code already keeps is cut off only where the attempt is the same. A different attempt still goes through the checker.
The stars of a message that goes without formatting (PLAIN, one short line) go as written, the same rule as the backticks.

1 done · 344 cases sorted · 183 tests passed · 225 of 228 passed · 9 files changed · 2 commits · 1 deployment

Uncommitted changes of earlier work were kept: docs/guide.md, docs/deploy.md, docs/web.md, scripts/deploy.ts, scripts/check.ts and 33 more` },
  { case: "reopened report", text: `The way to reopen a finished report with its earlier results kept is finished. A finished report can now be opened again from the command or the console, and what the earlier round did and its result documents stay, for the new plan to read and carry on.

What I fixed
I made the command run report --todo <number> --reopen and the console's "Open again" button on a report. Opening it again moves every finished step, with its result and document path, into history, named r1-plan, r1-step-0-0 and so on, and keeps that round's purpose and instruction apart. Then the report goes back to draft and the entry back to waiting.
Changes to the purpose, the instruction and the model are taken in the same request that opens it again. The moment an entry goes back to waiting the queue may pick it, so a change made after opening could have the plan start on the instruction from before.
The new round's building turn takes the earlier round's purpose, instruction and results as input, with the word not to search again what still holds. Each step's earlier result carries its first 2,000 characters only and points to its document for the rest. On a copy of the record the planning prompt went from 120,000 to 59,000 characters.
Units, side workers, prompts and result document names carry the round (r2-plan and so on), so no earlier document is overwritten and no row or launch count is mixed. The console's tag reads "round 2 step 1-1" too.
When the command refuses a finished report it now points to --reopen. The documents guide.md and web.md and the manual are fixed, and the manual is reloaded.

What I checked
I added tests. Running one round to the end and opening it again puts the earlier study, build and final results and the old purpose and instruction into the new planning prompt. Names stand as r2- and the earlier documents stay. There are tests for the control API's reopening and the console panel as well.
The typecheck passed at the root and in the web, and the 176 tests I touched passed. Of the whole suite's 222 files 221 passed. The one that failed is graph.test, on new files another job has not committed. On a tree holding my commits alone, the typecheck and 174 tests passed.
I ran the script's --reopen on a copy of the live database. On a development console started on that copy and a stand-in control API, reopening by the button worked.
After the deployment the service came up healthy, and the real control API refused reopening an unfinished report with 409. The new console bundle carries the button.

What I left
The uncommitted changes of another job are untouched; I picked my own hunks to commit. No migration changes the schema. The temporary login was made in the copy only and went with it. The original report itself was not opened again.

1 done · 0 open · 176 touched tests passed · 221/222 files passed (the 1 failure is another job's) · 18 files changed

Uncommitted changes of earlier work were kept: docs/guide.md, docs/deploy.md, docs/web.md, scripts/deploy.ts, scripts/check.ts and 32 more` },
  { case: "runner records", text: `This run spent no tokens of its own.

The "four checkouts" in the alert are temporary folders named tmp-run-…-checkout. The overnight job ran its tests just before deploying (06:15, 06:21, 06:24), and the fake runner inside the tests wrote fake session records under the state folder. The watcher read those records as real sessions.

There are three reasons.
① runner.test.ts uses a fake engine and a fake runner instead of the real ones, and is meant to remove the runner- records when it ends.
② The "usage limit reached" line in the watcher's log is the same as the test phrase in queue.test.ts and runner.test.ts.
③ The usage record shows no such burst at that time.

What was really spent in the same window (05:50~06:30):
overnight job: 18.3M tokens, about $13, mostly cache reads
earlier job: 14.8M tokens, about $11
the chat agent: 25 turns, 12.5M tokens

The same false alert came at 05:54 as well. It looks to have begun when the tests started imitating the runner's records, and it will ring again each time the tests run. Shall I file an entry to have the tests write their fake records outside the state folder, or the watcher pass over this temporary folder?` },
  { case: "editor models", text: `I added the requester's words, as they were, to the entry. It is the work that changes where an editor's model and harness are chosen, so it went there, and nothing has run for it yet, so they reach the next run's prompt as they are.

This is how the editors run now.
The digest and the writer are both tied to one editor line and run on one model. The writer is on high effort and keeps its session.
The gates have a model set per agent, mostly a local model, and take one request and end.

I read "a single agent like the gates" this way:
① Each agent can have its own model.
② It takes one request and ends instead of keeping a session.
③ The default is a mid model, matched by a model of the same level on the other providers.

Once ② is done, the session problem carried over from the earlier entry is solved and running it on another machine gets easier too. If it was not meant to go as far as running without a session, say so.` },
  { case: "two readers", text: `The two processes read different material and write to different places.

① The digest
Material: the notes the agent left with note.ts
What it does: sorts each note into one of three. Written as a room's rule it goes into the rules folder, written as knowledge into the knowledge folder, and neither is dropped. It runs every hour, and at once when the requester says "sort it now".

② The writer
Material: a room's conversation as written, one batch at a time
What it does: writes pieces by subject for each speaker, such as character, way of speaking and relations, into the people folder and keeps the index in step. Then the summariser makes the standing summary out of those pieces.

The digest sorts the notes the agent wrote into rules or knowledge, and the writer reads the conversation itself and writes about the people.

The digest's instruction already has the knowledge documents. It tags a note as one room's own or shared, and keeps shared only what the requester said may be shared. Making the new knowledge documents and the knowledge sorter may overlap with this.` },
  { case: "login note", text: `Log in to the host over SSH and type \`relay\`, and that is it.

\`\`\`
ssh user@example-host
/usr/local/bin/relay
\`\`\`

The relay service keeps \`relay server\` up as the default session, on the default settings path, so typing it with no argument joins that session. Where /usr/local/bin is not on the PATH, type the whole path as above.

Before joining, \`relay status\` shows whether the server is up.
To leave, press \`ctrl+b\` then \`q\` — the window alone goes, and the jobs running inside stay alive.

To join straight from another computer without SSH, \`relay --remote user@example-host\` works too. That computer needs relay installed as well.` }
];

describe("answers shaped as long reports", () => {
  for (const one of corpus) {
    test(`${one.case} converts with sound tags and no marker beside a tag`, () => {
      const converted = convertMarkdown(one.text);

      expectClean(converted.markup);
      expect(visibleText(converted.markup)).toBe(converted.plain);

      const folded = foldedMarkup(converted);

      if (folded !== null) {
        expectClean(folded);
        expect(visibleText(folded)).toBe(converted.plain);
      }
    });
  }

  test("the login note's code block and backticks become pre and code, and the plain text keeps none of them", () => {
    const converted = convertMarkdown(corpus.find((one) => one.case === "login note")?.text ?? "");

    expect(converted.markup).toContain("<pre>ssh user@example-host\n/usr/local/bin/relay</pre>");
    expect(converted.markup).toContain("type <code>relay</code>, and");
    expect(converted.plain).not.toContain("`");
  });

  test("the reopened report's angle brackets are escaped, not read as a tag", () => {
    const converted = convertMarkdown(corpus.find((one) => one.case === "reopened report")?.text ?? "");

    expect(converted.markup).toContain("--todo &lt;number&gt; --reopen");
    expect(converted.plain).toContain("--todo <number> --reopen");
  });

  test("the plain report's stars that are no pair stay as they were written", () => {
    const converted = convertMarkdown(corpus.find((one) => one.case === "plain report")?.text ?? "");

    expect(converted.plain).toContain("2**3, server/**/*.ts");
    expect(converted.markup).not.toContain("<b>");
    expect(converted.markup).not.toContain("<i>");
  });
});

describe("what Telegram has no tag for is laid flat", () => {
  test("a heading keeps its words and loses its marks", () => {
    const converted = convertMarkdown("## Result\nThe body.\n### **Bold** title ###");

    expect(converted.markup).toBe("Result\nThe body.\n<b>Bold</b> title");
    expect(converted.plain).toBe("Result\nThe body.\nBold title");
  });

  test("bullets become •, nested ones keep their indent, and numbers stay as written", () => {
    const converted = convertMarkdown("- one\n  * two\n+ three\n1. first\n2) second");

    expect(converted.plain).toBe("• one\n  • two\n• three\n1. first\n2) second");
    expectClean(converted.markup);
  });

  test("a table is its rows with the cells joined, and the rule row is gone", () => {
    const converted = convertMarkdown("| name | value |\n|---|:---:|\n| `a` | 1 |\n| b | **2** |");

    expect(converted.markup).toBe("name · value\n<code>a</code> · 1\nb · <b>2</b>");
    expectClean(converted.markup);
  });

  test("a pipe inside a cell's code span or link address is not a cell border", () => {
    const converted = convertMarkdown("| command | what it does |\n|---|---|\n| `ls | grep foo` | the files holding foo |\n| [search](https://example.com/?q=a|b) | address |");

    expect(converted.markup).toBe(
      'command · what it does\n<code>ls | grep foo</code> · the files holding foo\n<a href="https://example.com/?q=a|b">search</a> · address',
    );
    expectClean(converted.markup);
  });

  test("pipes in a sentence are not a table", () => {
    expect(convertMarkdown("a | b, one of the two\nthe next line").plain).toBe("a | b, one of the two\nthe next line");
  });

  test("a horizontal rule is gone, and so is a setext underline", () => {
    expect(convertMarkdown("above\n\n---\n\nbelow").plain).toBe("above\n\nbelow");
    expect(convertMarkdown("Title\n---\nbody").plain).toBe("Title\nbody");
    expect(convertMarkdown("above\n***\nbelow").plain).toBe("above\nbelow");
  });

  test("an indented line is text, not code", () => {
    const converted = convertMarkdown("item\n    an indented note");

    expect(converted.markup).toBe("item\n    an indented note");
    expect(converted.formatted).toBe(false);
  });

  test("a text with no markdown is its own plain text, byte for byte, with no tags", () => {
    const text = "Soup would be good for dinner tonight.\n\nFor 3~5 people, order about 2*3 bowls.";
    const converted = convertMarkdown(text);

    expect(converted.plain).toBe(text);
    expect(converted.markup).toBe(text);
    expect(converted.formatted).toBe(false);
  });
});

describe("the tags a message's markdown becomes", () => {
  test("bold, italic, strike, spoiler and code", () => {
    const converted = convertMarkdown("**bold** *slanted* ~~struck~~ ||hidden|| `value`");

    expect(converted.markup).toBe("<b>bold</b> <i>slanted</i> <s>struck</s> <tg-spoiler>hidden</tg-spoiler> <code>value</code>");
    expect(converted.plain).toBe("bold slanted struck hidden value");
  });

  test("a code block keeps its language and escapes what it holds", () => {
    const converted = convertMarkdown("Look.\n```ts\nif (a < b && c) {}\n\n```\nThe end.");

    expect(converted.markup).toBe('Look.\n<pre><code class="language-ts">if (a &lt; b &amp;&amp; c) {}\n</code></pre>\nThe end.');
    expect(converted.plain).toBe("Look.\nif (a < b && c) {}\n\nThe end.");
    expectClean(converted.markup);
  });

  test("a fence that never closes runs to the end", () => {
    expect(convertMarkdown("```\n**not bold**").markup).toBe("<pre>**not bold**</pre>");
  });

  test("a code block with nothing in it is left out", () => {
    expect(convertMarkdown("above\n```\n```\nbelow").markup).toBe("above\nbelow");
  });

  test("< > & are escaped in text, in code and in an href", () => {
    const converted = convertMarkdown('a < b & c > d `x<y&z` [here](https://example.com/?a=1&b="2")');

    expect(converted.markup).toBe(
      'a &lt; b &amp; c &gt; d <code>x&lt;y&amp;z</code> <a href="https://example.com/?a=1&amp;b=&quot;2&quot;">here</a>',
    );
    expectClean(converted.markup);
  });

  test("HTML in the message is shown as its characters", () => {
    const converted = convertMarkdown("<div>title</div> one line<br>");

    expect(converted.markup).toBe("&lt;div&gt;title&lt;/div&gt; one line&lt;br&gt;");
    expect(converted.plain).toBe("<div>title</div> one line<br>");
  });

  test("a link needs an http address; any other stays words with its address after them", () => {
    expect(convertMarkdown("[press](javascript:alert(1))").markup).toBe("press javascript:alert(1)");
    expect(convertMarkdown("[map](https://map.example/x)").plain).toBe("map (https://map.example/x)");
    expect(convertMarkdown("<https://example.com/a>").markup).toBe("https://example.com/a");
    expect(convertMarkdown("![photo](https://example.com/p.png)").markup).toBe('<a href="https://example.com/p.png">photo</a>');
  });

  test("a bare address stays for Telegram to read unless the Korean sentence runs on straight after it", () => {
    expect(convertMarkdown("주소 https://site.example.com/archive/83 에 있어").formatted).toBe(false);
    expect(convertMarkdown("(https://site.example.com/archive/83)에 있어").markup).toBe('(<a href="https://site.example.com/archive/83">https://site.example.com/archive/83</a>)에 있어');
    expect(convertMarkdown("https://site.example.com/archive/83에").plain).toBe("https://site.example.com/archive/83에");
  });

  test("a quote is one blockquote, and a quote inside it is flattened into it", () => {
    const converted = convertMarkdown("> somebody's words\n>> the words inside them\n\nmy words");

    expect(converted.markup).toBe("<blockquote>somebody's words\nthe words inside them</blockquote>\n\nmy words");
    expectClean(converted.markup);
  });

  test("a formatting around code is split around it, since code may not sit inside one", () => {
    const converted = convertMarkdown("**before `value` after**");

    expect(converted.markup).toBe("<b>before </b><code>value</code><b> after</b>");
    expectClean(converted.markup);
  });

  test("code inside a link's words loses its tag and keeps its letters", () => {
    const converted = convertMarkdown("[`bun test` result](https://example.com/)");

    expect(converted.markup).toBe('<a href="https://example.com/">bun test result</a>');
    expectClean(converted.markup);
  });

  test("formatting inside formatting and inside a link nests the way Telegram takes it", () => {
    const converted = convertMarkdown("**bold *slanted inside* end** [*a slanted link*](https://example.com/)");

    expect(converted.markup).toBe('<b>bold <i>slanted inside</i> end</b> <a href="https://example.com/"><i>a slanted link</i></a>');
    expectClean(converted.markup);
  });
});

describe("markers that are no pair stay the letters they are", () => {
  test("glued to a letter, digit, underscore or slash, padded inside, or alone", () => {
    for (const text of ["2**3", "server/**/*.ts", "a || b", "3~5 people", "x*y*z", "**no pair", "a ** b ** c", "3~5~7"]) {
      expect(convertMarkdown(text).markup).toBe(text);
    }
  });

  test("underscores are never markup", () => {
    expect(convertMarkdown("snake_case_name and __init__.py").markup).toBe("snake_case_name and __init__.py");
  });

  test("a pair stuck to Korean on both sides is still a pair", () => {
    expect(convertMarkdown("이건**중요**합니다").markup).toBe("이건<b>중요</b>합니다");
  });

  test("a backslash takes a marker's meaning away and leaves other backslashes alone", () => {
    expect(convertMarkdown("\\*\\*star\\*\\* C:\\Users \\d").markup).toBe("**star** C:\\Users \\d");
  });

  test("stars inside backticks are the code's own letters", () => {
    expect(convertMarkdown("`**/*.ts`").markup).toBe("<code>**/*.ts</code>");
  });
});

describe("one bold per message, unless every bold is an item name", () => {
  test("bolds that each take a whole line all stand, bullets and numbers aside", () => {
    const converted = convertMarkdown("- **First**\na note\n① **Second**\na note\n**Third**\na note");

    expect(converted.markup.split("<b>")).toHaveLength(4);
  });

  test("bolds inside sentences leave the first one and the others as words", () => {
    const converted = convertMarkdown("**Before** a sentence.\n\n**Name**\nAnd **more** there.");

    expect(converted.markup).toBe("<b>Before</b> a sentence.\n\nName\nAnd more there.");
  });
});

const body = "body ".repeat(60).trim();
const conclusion = "The conclusion is this.";

describe("the fold, on blocks", () => {
  test("keeps the first block and a short last block in the open", () => {
    const folded = foldedMarkup(convertMarkdown(`An opening.\n\n${body}\n\n${conclusion}`));

    expect(folded).toBe(`An opening.\n\n<blockquote expandable>${body}</blockquote>\n\n${conclusion}`);
  });

  test("too little to hide is no fold", () => {
    expect(foldBlocks(convertMarkdown("An opening.\n\nOne line more.").document)).toBeNull();
    expect(foldBlocks(convertMarkdown("A short word of one paragraph.").document)).toBeNull();
  });

  test("a code block with a blank line inside is one block and is never cut", () => {
    const code = Array.from({ length: 12 }, (_, i) => `line ${i}`).join("\n");
    const text = `An opening.\n\n\`\`\`ts\n${code}\n\nmore\n\`\`\`\n\n${body}\n\n${conclusion}`;
    const folded = foldedMarkup(convertMarkdown(text)) ?? "";

    expect(folded).toStartWith("An opening.\n\n<blockquote expandable><pre><code class=\"language-ts\">line 0");
    expect(folded).toContain("\n\nmore</code></pre>\n\nbody");
    expectClean(folded);
  });

  test("a quote inside the folded part stands without its own quote", () => {
    const text = `An opening.\n\n> quoted words ${body}\n\n${body}\n\n${conclusion}`;
    const folded = foldedMarkup(convertMarkdown(text)) ?? "";

    expect(folded.match(/<blockquote/g)).toHaveLength(1);
    expectClean(folded);
  });

  test("a conclusion written onto the body with one line break stays out of the quote", () => {
    const text = `An opening.\n\n${body}\n${body}\nTo end, I will sum it up this way.`;
    const folded = foldedMarkup(convertMarkdown(text)) ?? "";

    expect(folded).toEndWith("</blockquote>\nTo end, I will sum it up this way.");
  });

  test("a last line that is not a finished sentence is an item, and stays where it is", () => {
    const fold = foldBlocks(convertMarkdown(`An opening.\n\n${body}\n13 a right-click context menu in the chat\n14 the TODO list into the database, and the room's agent asking too`).document);

    expect(fold?.closing).toBeNull();
  });

  test("a code block is never the conclusion", () => {
    const fold = foldBlocks(convertMarkdown(`An opening.\n\n${body}\n\n\`\`\`\nls\n\`\`\``).document);

    expect(fold?.closing).toBeNull();
  });
});

describe("the fold a send path makes without the gate", () => {
  test("leaves a short message as its plain text", () => {
    expect(codeMarkup(convertMarkdown("a short message"))).toBeNull();
  });

  test("sends a short message's formatting unfolded", () => {
    expect(codeMarkup(convertMarkdown("a short `value` message"))).toBe("a short <code>value</code> message");
  });

  test("folds a long one and escapes both halves", () => {
    const folded = codeMarkup(convertMarkdown(`An opening a < b\n\n${"body ".repeat(200)}`)) ?? "";

    expect(folded).toContain("An opening a &lt; b\n\n<blockquote expandable>");
  });
});

describe("a text longer than one message", () => {
  test("is divided at a pause, and the parts put back together are the text", () => {
    const text = Array.from({ length: 30 }, (_, i) => `paragraph ${i} **bold** ${"n".repeat(40)}`).join("\n\n");
    const parts = markdownParts(text, 300);

    expect(parts.length).toBeGreaterThan(1);
    expect(parts.join("")).toBe(text);

    for (const part of parts) {
      const converted = convertMarkdown(part);

      expect(Math.max(visibleText(converted.markup).length, converted.plain.length)).toBeLessThanOrEqual(300);
      expectClean(converted.markup);
    }
  });

  test("a code block across the boundary is closed in one part and opened again in the next", () => {
    const code = Array.from({ length: 40 }, (_, i) => `const value${i} = ${i} < 10 && ok;`).join("\n");
    const text = `An explanation.\n\n\`\`\`ts\n${code}\n\`\`\`\n\nThe end.`;
    const parts = markdownParts(text, 400);

    expect(parts.length).toBeGreaterThan(2);

    for (const part of parts) {
      const converted = convertMarkdown(part);

      expect(Math.max(visibleText(converted.markup).length, converted.plain.length)).toBeLessThanOrEqual(400);
      expectClean(converted.markup);
      expect((part.match(/^```/gm) ?? []).length % 2).toBe(0);
    }

    expect(convertMarkdown(parts[1] ?? "").markup).toStartWith('<pre><code class="language-ts">');
    expect(parts.map((part) => convertMarkdown(part).plain).join("")).toContain("const value39 = 39 < 10 && ok;");
  });

  test("a line cut at a space never cuts through a code span or a bold", () => {
    for (const [open, close, tag] of [["`", "`", "<code>"], ["**", "**", "<b>"]] as const) {
      const text = `${"a ".repeat(150)}${open}${"v ".repeat(99)}v${close} The end.`;
      const parts = markdownParts(text, 400);

      expect(parts.length).toBe(2);

      for (const part of parts) {
        const converted = convertMarkdown(part);

        expect(converted.plain).not.toContain(open);
        expectClean(converted.markup);
      }

      expect(convertMarkdown(parts[1] ?? "").markup).toStartWith(tag);
    }
  });
});

// A conversion is compared as its markup, plain text and whether it is formatted, and the fold and the two
// markups made from one are each given the source they are made from.
holdAll("markdown", cases, {
  parseInline: (text: string) => parseInline(text),
  parseMarkdown,
  renderInline,
  plainInline,
  convertMarkdown: (source: string) => {
    const { markup, plain, formatted } = convertMarkdown(source);

    return { markup, plain, formatted };
  },
  foldBlocks: (source: string) => foldBlocks(parseMarkdown(source)),
  foldedMarkup: (source: string) => foldedMarkup(convertMarkdown(source)),
  codeMarkup: (source: string) => codeMarkup(convertMarkdown(source)),
  markdownParts,
});
