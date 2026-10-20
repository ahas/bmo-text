import { shownPart, translationParts } from "../translation-parts";
import cases from "../../cases/translation-parts.json";
import { holdAll } from "./shared";

holdAll("translation-parts", cases, { translationParts, shownPart });
