// Connector kinds come from the server's registry in English; the page
// shows these instead when it knows the kind. Keys are `kinds.<name>.<field>`.
import type { Dict } from "./index";

export const en: Dict = {};

export const hr: Dict = {
  "kinds.gmail.label": "Gmail / Google Workspace",
  "kinds.gmail.description": "Računi i potvrde koje stižu kao PDF privici.",
  "kinds.mojeracun.label": "Moj-eRačun",
  "kinds.mojeracun.description":
    "Ulazni e-računi tvrtke u Moj-eRačunu: UBL kakav je dostavljen, njegov ugrađeni PDF te dobavljač, broj, datum i iznos pročitani iz njega.",
  "kinds.mojeracun.consent_note":
    "API korisnik i lozinka tvrtke u Moj-eRačunu, njezin OIB, poslovna jedinica ako je registrirana i SoftwareId koji je Moj-eRačun izdao integratoru (integracije@moj-eracun.hr). Samo čitanje: dokumenti se preuzimaju, ništa se ne potvrđuje, ne šalje ni ne mijenja u Moj-eRačunu.",
  "kinds.eracuni.label": "e-računi",
  "kinds.eracuni.description":
    "Primljeni računi i sandučić e-računa tvrtke kod posrednika e-računi, s dobavljačem, brojem, datumom i iznosom kako ih e-računi zna.",
  "kinds.eracuni.consent_note":
    "Vaš API korisnik u e-računima, njegov API tajni ključ i token web servisa tvrtke (e-računi: Postavke → Web servisi). Samo čitanje: ništa se ne stvara ni ne mijenja u e-računima.",
  "kinds.gmail.consent_note":
    "Privola prati svrhu. Čitanje je samo za čitanje: ništa se ne premješta ni briše, a čitaju se samo poruke koje odgovaraju upitu (PDF privici se čuvaju, račun bez privitka ispisuje se na stranicu). Slanje je samo dopuštenje za slanje: sandučić povezan samo za slanje nikad se ne čita, ne popisuje i ne povlači.",
};
