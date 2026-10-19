import { withoutDashes } from "../dashes";
import cases from "../../cases/dashes.json";
import { holdAll } from "./shared";

holdAll("dashes", cases, { withoutDashes: (text: string, wrapped: boolean) => withoutDashes(text, { wrapped }) });
