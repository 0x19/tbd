// The dictionaries, one file per page or shared piece. Every file exports
// `en` and `hr` maps keyed `<namespace>.<slug>`. Facts (the record, the
// projects, the playgrounds) are not here: they live in `src/data/site.ts`
// with their Croatian in `src/data/site.hr.ts`, and `useSite()` picks.
import * as about from "./about";
import * as common from "./common";
import * as contact from "./contact";
import * as home from "./home";
import * as legal from "./legal";
import * as playgrounds from "./playgrounds";
import * as projects from "./projects";
import * as terms from "./terms";

export type Lang = "en" | "hr";
export const LANGS: { value: Lang; label: string }[] = [
  { value: "en", label: "English" },
  { value: "hr", label: "Hrvatski" },
];

export type Dict = Record<string, string>;

const all = [common, home, about, playgrounds, projects, contact, legal, terms];

export const messages: Record<Lang, Dict> = {
  en: Object.assign({}, ...all.map((m) => m.en)) as Dict,
  hr: Object.assign({}, ...all.map((m) => m.hr)) as Dict,
};
