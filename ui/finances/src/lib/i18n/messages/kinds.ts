// Connector kinds come from the server's registry in English; the page
// shows these instead when it knows the kind. Keys are `kinds.<name>.<field>`.
import type { Dict } from "./index";

export const en: Dict = {};

export const hr: Dict = {
  "kinds.gmail.label": "Gmail / Google Workspace",
  "kinds.gmail.description": "Računi i potvrde koje stižu kao PDF privici.",
  "kinds.eracuni.label": "e-računi",
  "kinds.eracuni.description":
    "Primljeni računi i sandučić e-računa tvrtke kod posrednika e-računi, s dobavljačem, brojem, datumom i iznosom kako ih e-računi zna.",
  "kinds.eracuni.consent_note":
    "Vaš API korisnik u e-računima, njegov API tajni ključ i token web servisa tvrtke (e-računi: Postavke → Web servisi). Samo čitanje: ništa se ne stvara ni ne mijenja u e-računima.",
  "kinds.gmail.consent_note":
    "Privola prati svrhu. Čitanje je samo za čitanje: ništa se ne premješta ni briše, a čitaju se samo poruke koje odgovaraju upitu (PDF privici se čuvaju, račun bez privitka ispisuje se na stranicu). Slanje je samo dopuštenje za slanje: sandučić povezan samo za slanje nikad se ne čita, ne popisuje i ne povlači.",
};
