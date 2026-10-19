import { escapeAttribute, escapeHtml, expandable, expandableMarkup, faultReason, isLinkable, markupFault, visibleText } from "../format";
import cases from "../../cases/format.json";
import { holdAll } from "./shared";

holdAll("format", cases, { escapeHtml, escapeAttribute, isLinkable, visibleText, expandableMarkup, expandable, markupFault, faultReason });
