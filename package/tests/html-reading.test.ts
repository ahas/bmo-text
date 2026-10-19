import { readHtml } from "../html-reading";
import cases from "../../cases/html-reading.json";
import { holdAll } from "./shared";

// The cases write a picture's bytes as base64, which is how a reading is compared with them.
holdAll("html-reading", cases, {
  readHtml: (html: string, stem: string) => {
    const reading = readHtml(html, stem);

    return { ...reading, pictures: reading.pictures.map((picture) => ({ ...picture, bytes: Buffer.from(picture.bytes).toString("base64") })) };
  },
});
