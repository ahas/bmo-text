// The ceiling every verb of ours answers under, and the one wording for saying what was left. The
// wording is English, since every reader of these answers is an agent; a caller names its unit as a
// plural noun and says how to ask for the rest as a sentence. A cut nobody is told about is the one
// shape this must not take, because a caller reading a short answer as the whole answer is worse off
// than one who was refused. The cut is the crate's, through the package's WebAssembly build.

import { binding } from "./binding";

/**
 * The most characters one of our verbs answers with before it cuts. It is set above what any ordinary
 * answer weighs rather than at what is comfortable, so nothing legitimate meets the ceiling.
 */
export const maxAnswerChars = binding.maxAnswerChars() as 30_000;

/** How much of a single long value stands before it is cut, where one value is the whole answer. */
export const maxValueChars = binding.maxValueChars() as 2_000;

/** What a verb answered with, and what it had to leave out to stay under the ceiling. */
export interface Capped<T> {
  /** The things that fit, in the order they were given. */
  kept: readonly T[];
  /** How many did not. Nought where the whole answer fitted. */
  left: number;
  /** The one line saying so, or nothing where nothing was left out. */
  note: string | null;
}

/**
 * The longest run of `items` from the front whose rendering fits under the ceiling, counted in code
 * points, with what was left out named in the caller's own terms.
 *
 * `render` is given a prefix and returns exactly what the verb would print for it, so the ceiling
 * is measured against the answer itself rather than against a guess at it. It is called a handful of
 * times, never once per item.
 *
 * Where even one item is over the ceiling, nothing is kept: half a row is not a row, and an answer
 * cut inside one item would be neither the thing asked for nor valid to parse.
 */
export function capItems<T>(
  items: readonly T[],
  render: (kept: readonly T[]) => string,
  said: { unit: string; how: string; limit?: number },
): Capped<T> {
  const prefix = (length: number): string => render(length === items.length ? items : items.slice(0, length));
  const cut: { kept: number; left: number; note: string | null } = binding.capItems(
    items.length,
    prefix,
    said.unit,
    said.how,
    said.limit,
  );

  return { kept: cut.left === 0 ? items : items.slice(0, cut.kept), left: cut.left, note: cut.note };
}

/**
 * One long value cut where it stands, with how much was cut said in place of the rest, counting code points
 * and never tearing a surrogate pair. What this is for is the value that is itself the answer, read as
 * prose rather than parsed, so the mark belongs in the line.
 */
export function capValue(text: string, limit: number = maxValueChars): string {
  return binding.capValue(text, limit);
}
