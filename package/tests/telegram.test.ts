import { botIdFromToken, plainParts, transcriptParts } from "../telegram";
import cases from "../../cases/telegram.json";
import { holdAll } from "./shared";

holdAll("telegram", cases, { plainParts, transcriptParts, botIdFromToken });
