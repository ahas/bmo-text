import { readFileSync } from "node:fs";
import { join } from "node:path";

import { binding } from "./binding";

// The wording check (`WORDING.md` by default): the glossary of purified words that must not be used, read out
// of that document, and the reading of a text for every use it does not allow. The readings are the crate's,
// through the package's WebAssembly build; a term given back is the caller's own object from the list it
// passed.

export const wordingDocument = binding.wordingDocument() as "WORDING.md";

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

// The codes a line's language can be told in, by the script it is written in.
export const knownLanguages: readonly string[] = binding.knownLanguages();

export function lineLanguage(line: string): string {
  return binding.lineLanguage(line);
}

// Reads the lists out of the document's own sections. A bullet in any of them that has the shape of
// an entry but names a language the check cannot tell is refused rather than skipped, so a typo
// never quietly switches an entry off; so is a phrase of a term's own sense that names no banned
// term, or does not hold the term it keeps.
export function parseGlossary(text: string): Glossary {
  return binding.parseGlossary(text);
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
  const done: { text: string; added: number[] } = binding.addBannedTerms(text, JSON.stringify(entries));

  return { text: done.text, added: done.added.map((at) => entries[at]!) };
}

// Every use of a banned term in the given text, on the lines of the term's own language.
export function findBanned(path: string, text: string, banned: readonly BannedTerm[]): WordingHit[] {
  const hits: { line: number; text: string; term: number }[] = binding.findBanned(text, JSON.stringify(banned));

  return hits.map((hit) => ({ path, line: hit.line, text: hit.text, term: banned[hit.term]! }));
}

// The banned terms a text being written uses outside its quoted spans, each once and in the
// glossary's order. The allowed uses are the file check's and are not read here: they name files.
export function bannedTermsIn(text: string, glossary: Glossary): BannedTerm[] {
  const used: number[] = binding.bannedTermsIn(text, JSON.stringify(glossary.banned));

  return used.map((at) => glossary.banned[at]!);
}

// The terms in one line, each beside what to write: `"<term>" → <what to write>; …`.
export function bannedTermsLine(terms: readonly BannedTerm[]): string {
  return binding.bannedTermsLine(JSON.stringify(terms));
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
  const report: { hits: { path: string; line: number; text: string; term: number }[]; unusedAllowances: number[] } =
    binding.checkWording([...paths], read, JSON.stringify(glossary));

  return {
    hits: report.hits.map((hit) => ({ path: hit.path, line: hit.line, text: hit.text, term: glossary.banned[hit.term]! })),
    unusedAllowances: report.unusedAllowances.map((at) => glossary.allowed[at]!),
  };
}
