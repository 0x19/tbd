// The chat dock at the bottom of every page (RFC 0011). Keys are `dock.<slug>`.
import type { Dict } from "./index";

export const en: Dict = {
  // An agent's name by its id; one missing here shows the name the service gives.
  "dock.name.site": "Site guide",
  "dock.ask_page": "about this page, or anything on the site",
  "dock.open": "open the chat",
  "dock.close": "close",
  "dock.clear": "clear",
  "dock.continue": "continue in the workbench →",
  "dock.placeholder": "Ask about this page or the site",
  "dock.page": "reading {path}",
  "dock.keys": "enter send · esc close · / or ctrl+k open",
  "dock.example.1": "What is this page about?",
  "dock.example.2": "What is being built in the lab right now?",
  "dock.example.3": "Who is the site about, and what is the work?",
  "dock.unavailable": "The site guide is not answering right now.",
  "dock.admins": "admins only while the lab is private",
  "dock.sends.title": "What this sends",
  "dock.sends":
    "Your question, the earlier turns of this chat and the path of the page you are on go to the model service on this domain as your account. The service speaks as the site guide: it puts the guide's instructions and what it knows about the site in front of your turns. It records who asked, which agent, tier and model answered, the tokens and how it ended, for your daily budget; it does not store the question or the answer. This chat lives in this browser only.",
};

export const hr: Dict = {
  "dock.name.site": "Vodič kroz web",
  "dock.ask_page": "o ovoj stranici ili bilo čemu na webu",
  "dock.open": "otvori razgovor",
  "dock.close": "zatvori",
  "dock.clear": "očisti",
  "dock.continue": "nastavi na radnom stolu →",
  "dock.placeholder": "Pitaj o ovoj stranici ili webu",
  "dock.page": "čitaš {path}",
  "dock.keys": "enter pošalji · esc zatvori · / ili ctrl+k otvori",
  "dock.example.1": "O čemu je ova stranica?",
  "dock.example.2": "Što se trenutno gradi u laboratoriju?",
  "dock.example.3": "O kome je ovaj web i čime se bavi?",
  "dock.unavailable": "Vodič kroz web trenutno ne odgovara.",
  "dock.admins": "samo za administratore dok je laboratorij privatan",
  "dock.sends.title": "Što se šalje",
  "dock.sends":
    "Tvoje pitanje, raniji potezi ovog razgovora i putanja stranice na kojoj si idu servisu modela na ovoj domeni kao tvoj račun. Servis govori kao vodič kroz web: ispred tvojih poruka stavlja vodičeve upute i ono što zna o webu. Bilježi tko je pitao, koji su agent, razina i model odgovorili, tokene i kako je završilo, radi tvog dnevnog proračuna; ne pohranjuje ni pitanje ni odgovor. Ovaj razgovor živi samo u ovom pregledniku.",
};
