// The lab: the index, one RFC or study, and the demo's page. Keys are `lab.<slug>`.
// The prose of every RFC and study is English in both languages; only this chrome translates.
import type { Dict } from "./index";

export const en: Dict = {
  "lab.eyebrow": "Lab",
  "lab.title": "What is being built, with the numbers.",
  "lab.lead":
    "RFCs before the code, studies after the measurements, and the demos in between. Everything here is in progress and says so; a stamp on every page tells you how settled it is.",
  "lab.english_only": "The RFCs and studies are written in English in both languages.",
  "lab.live.label": "Live",
  "lab.live.lead": "Things you can poke at now, running on the same platform the pages describe.",
  "lab.rfcs.label": "RFCs",
  "lab.rfcs.lead":
    "What was decided before it was built, and why. Argued in the open; superseded, never rewritten.",
  "lab.studies.label": "Studies",
  "lab.studies.lead": "What was measured, what surprised us, and what we would do differently.",
  "lab.empty.rfcs": "No RFC is public yet.",
  "lab.empty.studies": "No study is public yet. The first one follows the first RFC.",
  "lab.status.open": "open",
  "lab.status.decided": "decided",
  "lab.status.superseded": "superseded",
  "lab.status.running": "running",
  "lab.status.measured": "measured",
  "lab.status.published": "published",
  "lab.rfc": "RFC",
  "lab.study": "Study",
  "lab.supersedes": "Supersedes",
  "lab.superseded_by": "Superseded by",
  "lab.measures": "Measures",
  "lab.back": "Back to the lab",
  "lab.demo.eyebrow": "Lab · demo",
  "lab.demo.title": "Ask the platform",
  "lab.demo.tag": "Soon",
  "lab.demo.card":
    "A model served from one workstation, answering questions about this platform, while the chaos tool hammers it and the numbers stay on screen. Behind a sign-in.",
  "lab.demo.lead":
    "A model served from one workstation, answering questions about this platform, while the chaos tool hammers it and the numbers stay on screen.",
  "lab.demo.gated.label": "Behind a sign-in",
  "lab.demo.gated":
    "Anonymous model endpoints get farmed within a day, so asking needs an account, and every account has a daily token budget. Signed out, you will see the last session's transcript and the live figures; signed in, you can ask and start a run yourself.",
  "lab.demo.signin": "Sign in on the gated site",
  "lab.demo.sends.label": "What this page sends",
  "lab.demo.sends.text":
    "Nothing yet. This page is a description. When the demo is live, what you type is sent to the model service on this domain and recorded with your account for the budget; the page will say so here, in this section, before it does.",
};

export const hr: Dict = {
  "lab.eyebrow": "Laboratorij",
  "lab.title": "Što se gradi, s brojkama.",
  "lab.lead":
    "RFC-ovi prije koda, studije nakon mjerenja, a demo između. Sve ovdje je u nastajanju i tako i piše; pečat na svakoj stranici kaže koliko je odluka zrela.",
  "lab.english_only": "RFC-ovi i studije pisani su na engleskom u oba jezika.",
  "lab.live.label": "Uživo",
  "lab.live.lead": "Stvari koje možeš bockati sada, na istoj platformi koju stranice opisuju.",
  "lab.rfcs.label": "RFC-ovi",
  "lab.rfcs.lead":
    "Što je odlučeno prije gradnje i zašto. Raspravljano javno; zamijenjeno novim, nikad prepisano.",
  "lab.studies.label": "Studije",
  "lab.studies.lead": "Što je izmjereno, što nas je iznenadilo i što bismo napravili drukčije.",
  "lab.empty.rfcs": "Nijedan RFC još nije javan.",
  "lab.empty.studies": "Nijedna studija još nije javna. Prva slijedi prvi RFC.",
  "lab.status.open": "otvoren",
  "lab.status.decided": "odlučen",
  "lab.status.superseded": "zamijenjen",
  "lab.status.running": "u tijeku",
  "lab.status.measured": "izmjereno",
  "lab.status.published": "objavljeno",
  "lab.rfc": "RFC",
  "lab.study": "Studija",
  "lab.supersedes": "Zamjenjuje",
  "lab.superseded_by": "Zamijenjen s",
  "lab.measures": "Mjeri",
  "lab.back": "Natrag u laboratorij",
  "lab.demo.eyebrow": "Laboratorij · demo",
  "lab.demo.title": "Pitaj platformu",
  "lab.demo.tag": "Uskoro",
  "lab.demo.card":
    "Model posluživan s jedne radne stanice odgovara na pitanja o ovoj platformi dok ga alat za kaos opterećuje, a brojke ostaju na ekranu. Iza prijave.",
  "lab.demo.lead":
    "Model posluživan s jedne radne stanice odgovara na pitanja o ovoj platformi dok ga alat za kaos opterećuje, a brojke ostaju na ekranu.",
  "lab.demo.gated.label": "Iza prijave",
  "lab.demo.gated":
    "Anonimne krajnje točke modela netko iscrpi u danu, pa je za pitanje potreban račun, a svaki račun ima dnevni proračun tokena. Bez prijave vidiš zapis zadnje sesije i brojke uživo; s prijavom možeš pitati i sam pokrenuti test.",
  "lab.demo.signin": "Prijavi se na zaštićenoj stranici",
  "lab.demo.sends.label": "Što ova stranica šalje",
  "lab.demo.sends.text":
    "Još ništa. Ova je stranica opis. Kad demo bude uživo, ono što upišeš šalje se servisu modela na ovoj domeni i bilježi uz tvoj račun radi proračuna; stranica će to reći ovdje, u ovom odjeljku, prije nego što to učini.",
};
