import { addressAsWritten, bareAddressAt, gluedAfterAddress, isWebAddress, markdownLinkAt, webAddresses } from "../web-address";
import cases from "../../cases/web-address.json";
import { holdAll } from "./shared";

holdAll("web-address", cases, { addressAsWritten, isWebAddress, markdownLinkAt, bareAddressAt, gluedAfterAddress, webAddresses });
