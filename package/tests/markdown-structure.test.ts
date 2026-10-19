import { halveText, headingSlug, markdownHeadings, markdownSections, plainHeading } from "../markdown-structure";
import cases from "../../cases/markdown-structure.json";
import { holdAll } from "./shared";

holdAll("markdown-structure", cases, { markdownHeadings, plainHeading, headingSlug, markdownSections, halveText });
