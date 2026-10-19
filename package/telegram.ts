// What one Telegram message holds, counted in UTF-16 units after the markup is parsed, and plain words too
// long for one message divided as a client divides a long paste. A longer text is never cut and never
// refused: it is divided into consecutive messages, and the parts put back together are the text exactly.
// The readings are the crate's, through the package's WebAssembly build.

import { binding } from "./binding";

// One message's text.
export const telegramTextLimit = binding.telegramTextLimit() as 4096;
// What a file's caption holds on an account without Premium, which is the only size a sender can count on.
export const telegramCaptionLimit = binding.telegramCaptionLimit() as 1024;
// The largest file the Bot API takes.
export const maxSendFileBytes: number = binding.maxSendFileBytes();

// The bot's own Telegram user id, which is the number before the colon of its token; nought where there is
// none.
export function botIdFromToken(token: string): number {
  return binding.botIdFromToken(token);
}

// Plain words divided into consecutive messages of at most `limit` UTF-16 units each, the way a Telegram
// client sends a long paste: each part ends just after the last pause that fits (a blank line, else a line
// break, else a space), so the pause stays with the words before it, and where none fits, after the last
// whole character a reader sees as one.
export function plainParts(text: string, limit: number): string[] {
  return binding.plainParts(text, limit);
}

// The reading a voice was spoken from, folded under the voice: its first part as the voice's caption, the
// rest as replies of one message each, every part an expandable quote of its escaped words. A part ends
// where the next whole character a reader sees as one would not fit, and only a character that alone exceeds
// a part is divided between its code points; nothing is dropped.
export function transcriptParts(text: string): string[] {
  return binding.transcriptParts(text);
}
