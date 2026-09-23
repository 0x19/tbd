// The dictionaries, one file per namespace. Every file exports `en` and `hr`
// maps keyed `<namespace>.<slug>`.
import * as cv from "./cv";
import * as nav from "./nav";

export type Lang = "en" | "hr";
export const LANGS: { value: Lang; label: string }[] = [
  { value: "en", label: "English" },
  { value: "hr", label: "Hrvatski" },
];

export type Dict = Record<string, string>;

const all = [nav, cv];

export const messages: Record<Lang, Dict> = {
  en: Object.assign({}, ...all.map((m) => m.en)) as Dict,
  hr: Object.assign({}, ...all.map((m) => m.hr)) as Dict,
};
