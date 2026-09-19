// The dictionaries, one file per namespace so each page's strings live
// beside each other and two people can add to different pages at once.
// Every file exports `en` and `hr` maps keyed `<namespace>.<slug>`.
import * as accountant from "./accountant";
import * as banking from "./banking";
import * as categories from "./categories";
import * as common from "./common";
import * as connectors from "./connectors";
import * as documents from "./documents";
import * as invoices from "./invoices";
import * as kinds from "./kinds";
import * as mail from "./mail";
import * as nav from "./nav";
import * as overview from "./overview";
import * as parties from "./parties";
import * as transactions from "./transactions";

export type Lang = "en" | "hr";
export const LANGS: { value: Lang; label: string }[] = [
  { value: "en", label: "English" },
  { value: "hr", label: "Hrvatski" },
];

export type Dict = Record<string, string>;

const all = [
  common,
  nav,
  kinds,
  overview,
  transactions,
  categories,
  banking,
  invoices,
  parties,
  documents,
  connectors,
  accountant,
  mail,
];

export const messages: Record<Lang, Dict> = {
  en: Object.assign({}, ...all.map((m) => m.en)) as Dict,
  hr: Object.assign({}, ...all.map((m) => m.hr)) as Dict,
};
