/* tslint:disable */
/* eslint-disable */

/**
 * The crate's `EnglishTest`, held for the package's class of the same name.
 */
export class EnglishTest {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * The test `try_new` makes, throwing the compiler's message for a pattern that does not compile.
     */
    constructor(set_aside: string, latin_per_hangul: number);
    passes(text: string): boolean;
}

/**
 * The document with the entries added, and the index among `entries` of each entry written.
 */
export function addBannedTerms(text: string, entries: string): any;

export function addressAsWritten(candidate: string): string;

/**
 * The index among `banned` of each term the text uses.
 */
export function bannedTermsIn(text: string, banned: string): any;

export function bannedTermsLine(terms: string): string;

export function bareAddressAt(text: string, at: number): string | undefined;

/**
 * The bot's user id as a number, as the package gives it.
 */
export function botIdFromToken(token: string): number;

/**
 * The cut of `length` items, `render` being given how many from the front a prefix holds and giving what the
 * caller prints for it: how many are kept, how many are left and the note. An error `render` throws is
 * thrown on once the cut has stopped calling it.
 */
export function capItems(length: number, render: Function, unit: string, how: string, limit?: number | null): any;

export function capValue(text: string, limit?: number | null): string;

/**
 * The report of the files, each read through `read`: each hit with its term's index among the glossary's
 * banned terms, and the index among its allowed uses of each one left unused. An error `read` throws is
 * thrown on.
 */
export function checkWording(paths: string[], read: Function, glossary: string): any;

export function codeMarkup(converted: string): string | undefined;

export function convertMarkdown(source: string): any;

export function englishFindingWords(): any;

export function escapeAttribute(value: string): string;

export function escapeHtml(text: string): string;

export function expandable(text: string): string;

export function expandableMarkup(markup: string): string;

export function faultReason(fault: string): string;

/**
 * Every hit in the text, its term the index of the term among `banned`.
 */
export function findBanned(text: string, banned: string): any;

export function findingExcerpt(texts: string[], limit: number): string;

export function findingMarkdown(text: string, words: string): string;

export function foldBlocks(document: string): any;

export function foldedMarkup(converted: string): string | undefined;

export function footnoteMarks(line: string): string;

export function gluedAfterAddress(text: string, end: number): boolean;

export function halveText(text: string): any;

export function headingSlug(text: string): string;

export function holdsKorean(text: string): boolean;

export function isLinkable(url: string): boolean;

export function isWebAddress(url: string): boolean;

export function knownLanguages(): string[];

export function lineLanguage(line: string): string;

export function longTextLength(): number;

export function markdownHeadings(source: string): any;

export function markdownLinkAt(text: string, at: number): any;

export function markdownParts(text: string, limit?: number | null): string[];

export function markdownSections(source: string): string[];

export function markupFault(markup: string): any;

export function maxAnswerChars(): number;

export function maxSendFileBytes(): number;

export function maxValueChars(): number;

export function parseGlossary(text: string): any;

/**
 * A line's pieces, and where each construct stood in UTF-16 units where `spans` asks for it.
 */
export function parseInline(text: string, spans: boolean): any;

export function parseMarkdown(source: string): any;

export function partKeys(): string[];

export function pictureCaption(captioned: string): string;

export function plainHeading(text: string): string;

export function plainInline(nodes: string): string;

export function plainParts(text: string, limit: number): string[];

/**
 * The reading in the package's shape, each picture's bytes the bytes themselves rather than a list of
 * numbers.
 */
export function readHtml(html: string, stem: string): any;

export function renderInline(nodes: string): string;

export function shownPart(source: string, text?: string | null): string;

export function telegramCaptionLimit(): number;

export function telegramTextLimit(): number;

export function textLanguage(text: string): string;

export function transcriptParts(text: string): string[];

export function translationPartTokens(): number;

export function translationParts(text: string, budget?: number | null): string[];

export function visibleText(markup: string): string;

export function webAddresses(text: string): any;

export function withoutDashes(text: string, wrapped: boolean): string;

export function wordingDocument(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_englishtest_free: (a: number, b: number) => void;
    readonly addBannedTerms: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly addressAsWritten: (a: number, b: number) => [number, number];
    readonly bannedTermsIn: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly bannedTermsLine: (a: number, b: number) => [number, number, number, number];
    readonly bareAddressAt: (a: number, b: number, c: number) => [number, number];
    readonly botIdFromToken: (a: number, b: number) => number;
    readonly capItems: (a: number, b: any, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number];
    readonly capValue: (a: number, b: number, c: number, d: number) => [number, number];
    readonly checkWording: (a: number, b: number, c: any, d: number, e: number) => [number, number, number];
    readonly codeMarkup: (a: number, b: number) => [number, number, number, number];
    readonly convertMarkdown: (a: number, b: number) => any;
    readonly englishFindingWords: () => any;
    readonly englishtest_new: (a: number, b: number, c: number) => [number, number, number];
    readonly englishtest_passes: (a: number, b: number, c: number) => number;
    readonly escapeAttribute: (a: number, b: number) => [number, number];
    readonly escapeHtml: (a: number, b: number) => [number, number];
    readonly expandable: (a: number, b: number) => [number, number];
    readonly expandableMarkup: (a: number, b: number) => [number, number];
    readonly faultReason: (a: number, b: number) => [number, number, number, number];
    readonly findBanned: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly findingExcerpt: (a: number, b: number, c: number) => [number, number];
    readonly findingMarkdown: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly foldBlocks: (a: number, b: number) => [number, number, number];
    readonly foldedMarkup: (a: number, b: number) => [number, number, number, number];
    readonly footnoteMarks: (a: number, b: number) => [number, number];
    readonly gluedAfterAddress: (a: number, b: number, c: number) => number;
    readonly halveText: (a: number, b: number) => any;
    readonly headingSlug: (a: number, b: number) => [number, number];
    readonly holdsKorean: (a: number, b: number) => number;
    readonly isLinkable: (a: number, b: number) => number;
    readonly isWebAddress: (a: number, b: number) => number;
    readonly knownLanguages: () => [number, number];
    readonly lineLanguage: (a: number, b: number) => [number, number];
    readonly longTextLength: () => number;
    readonly markdownHeadings: (a: number, b: number) => any;
    readonly markdownLinkAt: (a: number, b: number, c: number) => any;
    readonly markdownParts: (a: number, b: number, c: number, d: number) => [number, number];
    readonly markdownSections: (a: number, b: number) => [number, number];
    readonly markupFault: (a: number, b: number) => any;
    readonly maxAnswerChars: () => number;
    readonly maxSendFileBytes: () => number;
    readonly maxValueChars: () => number;
    readonly parseGlossary: (a: number, b: number) => [number, number, number];
    readonly parseInline: (a: number, b: number, c: number) => any;
    readonly parseMarkdown: (a: number, b: number) => any;
    readonly partKeys: () => [number, number];
    readonly pictureCaption: (a: number, b: number) => [number, number];
    readonly plainHeading: (a: number, b: number) => [number, number];
    readonly plainInline: (a: number, b: number) => [number, number, number, number];
    readonly plainParts: (a: number, b: number, c: number) => [number, number];
    readonly readHtml: (a: number, b: number, c: number, d: number) => any;
    readonly renderInline: (a: number, b: number) => [number, number, number, number];
    readonly shownPart: (a: number, b: number, c: number, d: number) => [number, number];
    readonly telegramCaptionLimit: () => number;
    readonly telegramTextLimit: () => number;
    readonly textLanguage: (a: number, b: number) => [number, number];
    readonly transcriptParts: (a: number, b: number) => [number, number];
    readonly translationPartTokens: () => number;
    readonly translationParts: (a: number, b: number, c: number, d: number) => [number, number];
    readonly visibleText: (a: number, b: number) => [number, number];
    readonly webAddresses: (a: number, b: number) => any;
    readonly withoutDashes: (a: number, b: number, c: number) => [number, number];
    readonly wordingDocument: () => [number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_drop_slice: (a: number, b: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
