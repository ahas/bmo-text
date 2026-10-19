// The ceiling every verb of ours answers under, and the one wording for saying what was left. The
// wording is English, since every reader of these answers is an agent; a caller names its unit as a
// plural noun and says how to ask for the rest as a sentence.
//
// A verb that serialises whatever it found answers with whatever it found: the whole code graph as
// JSON is 307,316 characters and one broad search of it is 188,846, and those land in the session of
// whoever asked and stay there for the rest of it. What the caller wanted was the answer; the rest
// is a cost nobody chose.
//
// So the answer is cut and the cut is said out loud, in one wording, by every verb: how many of the
// things asked for are in the answer, how many are not, and how to ask for the rest in that verb's
// own terms — a narrower word, an area, a `LIMIT`, a `--before-id`. A cut nobody is told about is
// the one shape this must not take, because a caller reading a short answer as the whole answer is
// worse off than one who was refused.

// A count as the answer writes it, with the thousands mark.
function countText(value: number): string {
  return value.toLocaleString("en-US");
}

/**
 * The most characters one of our verbs answers with before it cuts.
 *
 * It is set above what any ordinary answer weighs rather than at what is comfortable: over two
 * weeks of this repository's own job traffic the largest result any Claude Code session's shell
 * call returned was 29,261 characters and the 99th was 16,420, so an answer this size is already
 * the far end of normal use and nothing legitimate meets the ceiling.
 */
export const maxAnswerChars = 30_000;

/** How much of a single long value stands before it is cut, where one value is the whole answer. */
export const maxValueChars = 2_000;

/** What a verb answered with, and what it had to leave out to stay under the ceiling. */
export interface Capped<T> {
  /** The things that fit, in the order they were given. */
  kept: readonly T[];
  /** How many did not. Nought where the whole answer fitted. */
  left: number;
  /** The one line saying so, or nothing where nothing was left out. */
  note: string | null;
}

// Characters are counted as code points, the way every other size in this repository is, so a
// Korean answer is measured in the units a reader of it counts in.
function chars(text: string): number {
  return [...text].length;
}

/**
 * The longest run of `items` from the front whose rendering fits under the ceiling, with what was
 * left out named in the caller's own terms.
 *
 * `render` is given a prefix and returns exactly what the verb would print for it, so the ceiling
 * is measured against the answer itself — indentation, separators and all — rather than against a
 * guess at it. It is called a handful of times, never once per item.
 *
 * Where even one item is over the ceiling, nothing is kept: half a row is not a row, and an answer
 * cut inside one item would be neither the thing asked for nor valid to parse.
 */
export function capItems<T>(
  items: readonly T[],
  render: (kept: readonly T[]) => string,
  said: { unit: string; how: string; limit?: number },
): Capped<T> {
  const limit = said.limit ?? maxAnswerChars;

  if (chars(render(items)) <= limit) {
    return { kept: items, left: 0, note: null };
  }

  // The largest prefix that fits, found by halving. What is being searched is monotonic in the
  // count for every rendering these verbs have — a longer prefix is a longer answer — so the
  // answer this lands on is the largest one that fits.
  let fits = 0;
  let over = items.length;

  while (over - fits > 1) {
    const middle = Math.floor((fits + over) / 2);

    if (chars(render(items.slice(0, middle))) <= limit) {
      fits = middle;
    } else {
      over = middle;
    }
  }

  const kept = items.slice(0, fits);
  const left = items.length - fits;

  return {
    kept,
    left,
    note: fits === 0
      ? `-- one of the ${said.unit} alone is over the ${countText(limit)}-character ceiling, so nothing was given. ${said.how}`
      : `-- cut at the ${countText(limit)}-character ceiling: ${countText(fits)} of ${countText(items.length)} ${said.unit}. For the other ${countText(left)}: ${said.how}`,
  };
}

/**
 * One long value cut where it stands, with how much was cut said in place of the rest. What this is
 * for is the value that is itself the answer — a message's text, a column's contents — where the
 * caller is reading prose rather than parsing, so the mark belongs in the line.
 */
export function capValue(text: string, limit: number = maxValueChars): string {
  const held = [...text];

  return held.length <= limit ? text : `${held.slice(0, limit).join("")}…(+${countText(held.length - limit)} chars)`;
}
