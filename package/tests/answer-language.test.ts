import { holdsKorean } from "../answer-language";
import cases from "../../cases/answer-language.json";
import { holdAll } from "./shared";

holdAll("answer-language", cases, { holdsKorean });
