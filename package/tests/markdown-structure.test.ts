import {
  halveText,
  headingSlug,
  markdownHeadings,
  markdownSections,
  markdownStructure,
  plainHeading,
} from "../markdown-structure";
import cases from "../../cases/markdown-structure.json";
import { holdAll } from "./shared";

holdAll("markdown-structure", cases, {
  markdownHeadings,
  plainHeading,
  headingSlug,
  markdownSections,
  markdownStructure,
  halveText,
});
