// Everything the machine says to a chat session is English — the system lines, a quiet instruction
// and the result it carries, the summary a replaced session holds of
// its own past. English is the machine's language and not the room's, and a turn with nothing but
// the machine's language in front of it answers in it. So such a turn says what language the room
// is answered in, and every answer is held to it before it goes anywhere.

const hangul = /[가-힣]/u;

/** Whether a piece of text has any Korean in it at all. */
export function holdsKorean(text: string): boolean {
  return hangul.test(text);
}

