// Connector kinds come from the server's registry in English; the page
// shows these instead when it knows the kind. Keys are `kinds.<name>.<field>`.
import type { Dict } from "./index";

export const en: Dict = {};

export const hr: Dict = {
  "kinds.gmail.label": "Gmail / Google Workspace",
  "kinds.gmail.description": "Računi i potvrde koje stižu kao PDF privici.",
  "kinds.gmail.consent_note":
    "Pristup sandučiću samo za čitanje. Ništa se ne šalje, premješta ni briše; čitaju se samo poruke koje odgovaraju upitu, a čuvaju se samo njihovi PDF privici.",
};
