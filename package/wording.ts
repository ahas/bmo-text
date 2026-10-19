import { readFileSync } from "node:fs";
import { join } from "node:path";

// The wording check (`WORDING.md` by default): the glossary of purified words that must not be used,
// read out of that document, and the reading of a text for every use it does not allow.

export const wordingDocument = "WORDING.md";

export interface BannedTerm {
  lang: string;
  term: string;
  instead: string;
  // When the entry applies, where the line says so after its dash.
  when?: string;
  // The phrases in which the term keeps its own sense and passes, each holding the term, where the
  // term is banned only in the sense it was taken for.
  keeps?: string[];
}

export interface AllowedUse {
  lang: string;
  term: string;
  path: string;
  excerpt: string;
}

export interface Glossary {
  banned: BannedTerm[];
  allowed: AllowedUse[];
}

export interface WordingHit {
  path: string;
  line: number;
  text: string;
  term: BannedTerm;
}

// How the language of one line is told, by the script it is written in. An entry names its
// language by code, and a code with no way of telling its lines is a mistake in the glossary.
const lineLanguages: { lang: string; holds: (line: string) => boolean }[] = [
  { lang: "ko", holds: (line) => /[ᄀ-ᇿ㄰-㆏가-힯]/.test(line) },
  { lang: "ja", holds: (line) => /[぀-ヿ]/.test(line) },
  { lang: "en", holds: () => true },
];

export const knownLanguages: readonly string[] = lineLanguages.map((one) => one.lang);

export function lineLanguage(line: string): string {
  return lineLanguages.find((one) => one.holds(line))!.lang;
}

const bannedLine = /^- ([a-z]{2,3}) · (.+?) → (.+?)(?: — (.*))?$/;
const allowedLine = /^- ([a-z]{2,3}) · (.+?) · (\S+) · (.+) — .+$/;
const ownSenseLine = /^- ([a-z]{2,3}) · (.+?) · (.+?) — .+$/;

// Reads the lists out of the document's own sections. A bullet in any of them that has the shape of
// an entry but names a language the check cannot tell is refused rather than skipped, so a typo
// never quietly switches an entry off; so is a phrase of a term's own sense that names no banned
// term, or does not hold the term it keeps.
export function parseGlossary(text: string): Glossary {
  const banned: BannedTerm[] = [];
  const allowed: AllowedUse[] = [];
  let section = "";

  for (const line of text.split("\n")) {
    if (line.startsWith("## ")) {
      section = line.slice(3).trim();
      continue;
    }

    if (section === "Banned terms") {
      const match = bannedLine.exec(line);

      if (match !== null) {
        banned.push({
          lang: checkedLanguage(match[1]!, line),
          term: match[2]!,
          instead: match[3]!,
          ...(match[4] === undefined ? {} : { when: match[4] }),
        });
      }
    } else if (section === "Allowed uses") {
      const match = allowedLine.exec(line);

      if (match !== null) {
        allowed.push({ lang: checkedLanguage(match[1]!, line), term: match[2]!, path: match[3]!, excerpt: match[4]! });
      }
    } else if (section === "Uses in the word's own sense") {
      const match = ownSenseLine.exec(line);

      if (match !== null) {
        keptPhrase(banned, checkedLanguage(match[1]!, line), match[2]!, match[3]!, line);
      }
    }
  }

  return { banned, allowed };
}

function keptPhrase(banned: BannedTerm[], lang: string, term: string, phrase: string, line: string): void {
  const entry = banned.find((one) => one.lang === lang && one.term === term);

  if (entry === undefined || !phrase.includes(term)) {
    throw new Error(`${wordingDocument}: a phrase of a word's own sense must hold a banned term listed above it: ${line}`);
  }

  entry.keeps = [...(entry.keeps ?? []), phrase];
}

function checkedLanguage(lang: string, line: string): string {
  if (!knownLanguages.includes(lang)) {
    throw new Error(`${wordingDocument}: unknown language code "${lang}" in: ${line}`);
  }

  return lang;
}

// The glossary in the folder `root`, under its conventional name.
export function readGlossary(root: string): Glossary {
  return parseGlossary(readFileSync(join(root, wordingDocument), "utf8"));
}

// The document with new banned lines written after the last entry of its list, each in the list's
// own shape. A term the list already bans in that language is passed over rather than written twice,
// and a line that would not read back as the entry it was meant to be is refused, so nothing is
// written that the check would then read otherwise.
export function addBannedTerms(text: string, entries: readonly BannedTerm[]): { text: string; added: BannedTerm[] } {
  const held = parseGlossary(text).banned;
  const added: BannedTerm[] = [];

  for (const entry of entries) {
    if (![...held, ...added].some((one) => one.lang === entry.lang && one.term === entry.term)) {
      added.push(entry);
    }
  }

  if (added.length === 0) {
    return { text, added };
  }

  const lines = text.split("\n");
  const heading = lines.indexOf("## Banned terms");
  const next = lines.findIndex((line, index) => index > heading && line.startsWith("## "));

  if (heading === -1) {
    throw new Error(`${wordingDocument}: there is no "## Banned terms" list to add to`);
  }

  let end = next === -1 ? lines.length : next;

  while (end > heading + 1 && lines[end - 1]!.trim() === "") {
    end -= 1;
  }

  const written = added.map((entry) => `- ${entry.lang} · ${entry.term} → ${entry.instead}${entry.when ? ` — ${entry.when}` : ""}`);
  const result = [...lines.slice(0, end), ...written, ...lines.slice(end)].join("\n");
  const reread = parseGlossary(result).banned;

  for (const entry of added) {
    const back = reread.find((one) => one.lang === entry.lang && one.term === entry.term);

    if (back === undefined || back.instead !== entry.instead || back.when !== entry.when) {
      throw new Error(`${wordingDocument}: the line for "${entry.term}" would not read back as written`);
    }
  }

  return { text: result, added };
}

const hangulSyllable = /[가-힣]/;

// Whether the term stands on its own where it was found, rather than being the tail of a longer
// word. Korean writes a word as one run of syllables with no space inside it, so a term another
// syllable runs into from the left belongs to that other word, as a longer verb holds a shorter one.
// Nothing is asked of the right side, because a particle or an ending follows the term there and the word is
// still the term's own.
function standsAlone(line: string, at: number, term: BannedTerm): boolean {
  if (at === 0 || !hangulSyllable.test(term.term[0]!)) {
    return true;
  }

  return !hangulSyllable.test(line[at - 1]!);
}

// Whether the term, where it was found, is part of a phrase in which it keeps its own sense: a word
// banned where it stands for a position still passes where it means a seat left empty.
function keepsOwnSense(line: string, at: number, term: BannedTerm): boolean {
  return (term.keeps ?? []).some((phrase) => {
    const start = at - phrase.indexOf(term.term);

    return start >= 0 && line.startsWith(phrase, start);
  });
}

// Every use of a banned term in the given text, on the lines of the term's own language.
export function findBanned(path: string, text: string, banned: readonly BannedTerm[]): WordingHit[] {
  const hits: WordingHit[] = [];

  text.split("\n").forEach((line, index) => {
    const lang = lineLanguage(line);

    for (const term of banned) {
      if (term.lang !== lang) {
        continue;
      }

      let at = line.indexOf(term.term);

      while (at !== -1 && (!standsAlone(line, at, term) || keepsOwnSense(line, at, term))) {
        at = line.indexOf(term.term, at + 1);
      }

      if (at !== -1) {
        hits.push({ path, line: index + 1, text: line, term });
      }
    }
  });

  return hits;
}

// A span a written text names a wrong form or quotes a command or somebody's words in: inside
// backticks or a pair of quotation marks, on one line.
const quotedSpan = /`[^`\n]*`|"[^"\n]*"|'[^'\n]*'|“[^”\n]*”|‘[^’\n]*’|「[^」\n]*」|『[^』\n]*』/g;

// The banned terms a text being written uses outside its quoted spans, each once and in the
// glossary's order. The allowed uses are the file check's and are not read here: they name files.
export function bannedTermsIn(text: string, glossary: Glossary): BannedTerm[] {
  const bare = text.replace(quotedSpan, (span) => " ".repeat(span.length));
  const found = new Set(findBanned("", bare, glossary.banned).map((hit) => hit.term));

  return glossary.banned.filter((term) => found.has(term));
}

// The terms in one line, each beside what to write: `"<term>" → <what to write>; …`.
export function bannedTermsLine(terms: readonly BannedTerm[]): string {
  return terms.map((term) => `"${term.term}" → ${term.instead}`).join("; ");
}

export interface WordingReport {
  hits: WordingHit[];
  unusedAllowances: AllowedUse[];
}

// Checks the given files, each read through `read`, against the glossary: the uses no allowance
// covers, and the allowances that no longer cover anything.
export function checkWording(
  paths: readonly string[],
  read: (path: string) => string,
  glossary: Glossary,
): WordingReport {
  const used = new Set<AllowedUse>();
  const hits: WordingHit[] = [];

  for (const path of paths) {
    for (const hit of findBanned(path, read(path), glossary.banned)) {
      const allowance = glossary.allowed.find(
        (one) =>
          one.lang === hit.term.lang && one.term === hit.term.term && one.path === hit.path && hit.text.includes(one.excerpt),
      );

      if (allowance === undefined) {
        hits.push(hit);
      } else {
        used.add(allowance);
      }
    }
  }

  return { hits, unusedAllowances: glossary.allowed.filter((one) => !used.has(one)) };
}
