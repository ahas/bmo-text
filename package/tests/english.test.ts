import { EnglishTest, holdsNonEnglish, nonEnglishLetters } from "../english";
import cases from "../../cases/english.json";
import { holdAll } from "./shared";

holdAll("english", cases, {
  passes: (setAside: string, latinPerHangul: number, text: string) => new EnglishTest(setAside, latinPerHangul).passes(text),
  holdsNonEnglish,
  nonEnglishLetters,
});
