"use client";

// Two languages, one dictionary per namespace, no library. A key is
// `<namespace>.<slug>`; a missing Croatian string falls back to English and a
// missing English one to the key, so a gap shows as a key and is never silent.
//
// The first language is decided, in this order: a choice the visitor made
// with the toggle (one first-party cookie on the parent domain, so the gated
// site on cv.<domain> reads the same; the visitor's own storage as well, for a
// dev server with no domain); the country Cloudflare saw them from, which the
// site's own `/whereami` answers with (Croatia, Bosnia, Serbia and Montenegro
// read Croatian); the browser's language; English. Only a choice the visitor
// makes with the toggle is kept, so the country and the browser keep deciding
// for everyone else. `ui/cv/src/lib/i18n/index.tsx` is this file, on purpose.
import { createContext, type ReactNode, useCallback, useContext, useEffect, useMemo, useState } from "react";

import { type Lang, LANGS, messages } from "./messages";

export type { Lang };
export { LANGS };

/** The cookie and the storage key: two letters, nothing about the person. */
const KEY = "inorbit.lang";
const YEAR = 60 * 60 * 24 * 365;
/** Countries whose visitors read Croatian without asking. */
const CROATIAN_COUNTRIES = new Set(["HR", "BA", "RS", "ME"]);
/** Browser languages that read Croatian without asking. */
const CROATIAN_TAGS = ["hr", "bs", "sr"];

type Vars = Record<string, string | number>;

type Ctx = {
  lang: Lang;
  /** True once the first language has been decided on the client. */
  ready: boolean;
  setLang: (l: Lang) => void;
  t: (key: string, vars?: Vars) => string;
};

const LangContext = createContext<Ctx>({ lang: "en", ready: false, setLang: () => {}, t: (k) => k });

/** One string, interpolated: `{name}` in the text is filled from `vars`. */
export function translate(lang: Lang, key: string, vars?: Vars): string {
  const raw = messages[lang][key] ?? messages.en[key] ?? key;
  if (!vars) return raw;
  return raw.replace(/\{(\w+)\}/g, (m, k: string) => (k in vars ? String(vars[k]) : m));
}

function isLang(v: unknown): v is Lang {
  return v === "en" || v === "hr";
}

/**
 * The domain the cookie is set on: the registrable name (`inorbit.hr`, so
 * `www.` and `cv.` share it), or none on localhost and IP addresses.
 */
function cookieDomain(): string | null {
  const host = window.location.hostname;
  if (!host.includes(".") || /^[\d.]+$/.test(host) || host.endsWith(".localhost")) return null;
  return host.split(".").slice(-2).join(".");
}

function readCookie(): Lang | null {
  try {
    const m = document.cookie.match(/(?:^|;\s*)inorbit\.lang=([^;]*)/);
    const v = m?.[1];
    return isLang(v) ? v : null;
  } catch {
    return null;
  }
}

function writeCookie(l: Lang) {
  try {
    const domain = cookieDomain();
    const secure = window.location.protocol === "https:" ? "; Secure" : "";
    document.cookie = `${KEY}=${l}; Path=/; Max-Age=${YEAR}; SameSite=Lax${domain ? `; Domain=${domain}` : ""}${secure}`;
  } catch {
    // no document, or cookies refused: the storage below still holds it here
  }
}

function stored(): Lang | null {
  const fromCookie = readCookie();
  if (fromCookie) return fromCookie;
  try {
    const saved = window.localStorage.getItem(KEY);
    return isLang(saved) ? saved : null;
  } catch {
    return null;
  }
}

function fromBrowser(): Lang {
  const tags = [...(navigator.languages ?? []), navigator.language]
    .filter(Boolean)
    .map((t) => t.toLowerCase());
  return tags.some((t) => CROATIAN_TAGS.some((c) => t === c || t.startsWith(`${c}-`))) ? "hr" : "en";
}

async function fromCountry(): Promise<Lang | null> {
  try {
    const res = await fetch("/whereami", { cache: "no-store" });
    if (!res.ok) return null;
    const code = (await res.text()).trim().toUpperCase();
    if (!/^[A-Z]{2}$/.test(code)) return null;
    return CROATIAN_COUNTRIES.has(code) ? "hr" : "en";
  } catch {
    return null;
  }
}

export function LangProvider({ children }: { children: ReactNode }) {
  // Render English first on both server and client, then decide: a static
  // export cannot know the visitor up front.
  const [lang, setLangState] = useState<Lang>("en");
  const [ready, setReady] = useState(false);
  useEffect(() => {
    let cancelled = false;
    const choice = stored();
    if (choice) {
      setLangState(choice);
      setReady(true);
      return;
    }
    void fromCountry().then((byCountry) => {
      if (cancelled) return;
      setLangState(byCountry ?? fromBrowser());
      setReady(true);
    });
    return () => {
      cancelled = true;
    };
  }, []);
  useEffect(() => {
    try {
      document.documentElement.lang = lang;
    } catch {
      // no document
    }
  }, [lang]);
  const setLang = useCallback((l: Lang) => {
    setLangState(l);
    writeCookie(l);
    try {
      window.localStorage.setItem(KEY, l);
    } catch {
      // storage blocked; the cookie, or failing that the page, keeps the choice
    }
  }, []);
  const value = useMemo<Ctx>(
    () => ({ lang, ready, setLang, t: (key, vars) => translate(lang, key, vars) }),
    [lang, ready, setLang],
  );
  return <LangContext.Provider value={value}>{children}</LangContext.Provider>;
}

/** The current language, whether it is settled yet, and the setter. */
export function useLang() {
  const { lang, ready, setLang } = useContext(LangContext);
  return { lang, ready, setLang };
}

/** The translator for the current language. */
export function useT() {
  return useContext(LangContext).t;
}
