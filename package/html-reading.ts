// An HTML file read for what it says, so a document can be written out of it. The reading keeps the words in
// their order and takes every picture written into the page as data out as a file of its own, named where it
// stood. It is structural and nothing more: headings keep their level as markdown headings, a list item is a
// dash, every block starts a line, and scripts and styles are not words. The reading is the crate's, through
// the package's WebAssembly build.

import { binding } from "./binding";

export interface HtmlPicture {
  // The name it stands under in the reading, and the one it is kept by.
  name: string;
  type: string;
  bytes: Uint8Array;
}

export interface HtmlReading {
  title: string | null;
  // What the page says of itself in its `description` meta, where it says anything.
  description: string | null;
  // The page's words, a block to a line, with pictures as markdown pictures where they stood.
  lines: string[];
  // Every picture the page carried inside itself, in the order they stood.
  pictures: HtmlPicture[];
}

// The reading of one page. `stem` is what the pictures taken out of it are named after, numbered
// in the order they stood.
export function readHtml(html: string, stem = "picture"): HtmlReading {
  return binding.readHtml(html, stem);
}
