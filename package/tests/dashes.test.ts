import { readDashes, withoutDashes } from "../dashes";
import cases from "../../cases/dashes.json";
import { holdAll } from "./shared";

holdAll("dashes", cases, {
  withoutDashes: (text: string, wrapped: boolean) => withoutDashes(text, { wrapped }),
  readDashes: (text: string, wrapped: boolean) => readDashes(text, { wrapped }),
});
