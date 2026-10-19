// One Telegram message holds 4096 UTF-16 units of text, counted after the markup is parsed, so
// the words themselves are what is measured. A longer text is never cut and never refused: it is
// divided into consecutive messages. The division falls where a reader would pause — the last
// blank line inside the window, else the last line break, else the last space — and only a text
// with none of those is divided between grapheme clusters, so no character is ever broken in two.
// The parts put back together are the text exactly.
export const telegramTextLimit = 4096;
// What a file's caption holds on an account without Premium, which is the only size a sender
// can count on.
export const telegramCaptionLimit = 1024;
// The largest file the Bot API takes.
export const maxSendFileBytes = 50 * 1024 * 1024;
// The bot's own Telegram user id, which is the number before the colon of its token.
export function botIdFromToken(token: string): number {
  const id = Number.parseInt(token.split(":")[0] ?? "", 10);

  return Number.isNaN(id) ? 0 : id;
}

