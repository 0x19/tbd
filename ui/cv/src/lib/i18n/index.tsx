"use client";

// Two languages, one dictionary per namespace, no library. A key is
// `<namespace>.<slug>`; a missing Croatian string falls back to English and
// a missing English one to the key, so a gap shows as a key and is never
// silent. The choice persists per browser and follows the browser's
// language the first time.
import { createContext, type ReactNode, useCallback, useContext, useEffect, useMemo, useState } from "react";

import { type Lang, LANGS, messages } from "./messages";

export type { Lang };
export { LANGS };

const KEY = "cv-lang";

type Vars = Record<string, string | number>;

type Ctx = {
  lang: Lang;
  setLang: (l: Lang) => void;
  t: (key: string, vars?: Vars) => string;
};

const LangContext = createContext<Ctx>({ lang: "en", setLang: () => {}, t: (k) => k });

/** One string, interpolated: `{name}` in the text is filled from `vars`. */
export function translate(lang: Lang, key: string, vars?: Vars): string {
  const raw = messages[lang][key] ?? messages.en[key] ?? key;
  if (!vars) return raw;
  return raw.replace(/\{(\w+)\}/g, (m, k: string) => (k in vars ? String(vars[k]) : m));
}

function initial(): Lang {
  try {
    const saved = window.localStorage.getItem(KEY);
    if (saved === "en" || saved === "hr") return saved;
    return navigator.language.toLowerCase().startsWith("hr") ? "hr" : "en";
  } catch {
    return "en";
  }
}

export function LangProvider({ children }: { children: ReactNode }) {
  // Render English first on both server and client, then adopt the saved
  // choice: a static export cannot know the browser's language up front.
  const [lang, setLangState] = useState<Lang>("en");
  useEffect(() => {
    setLangState(initial());
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
    try {
      window.localStorage.setItem(KEY, l);
    } catch {
      // storage blocked; the choice lasts the session
    }
  }, []);
  const value = useMemo<Ctx>(
    () => ({ lang, setLang, t: (key, vars) => translate(lang, key, vars) }),
    [lang, setLang],
  );
  return <LangContext.Provider value={value}>{children}</LangContext.Provider>;
}

/** The current language and the setter. */
export function useLang() {
  const { lang, setLang } = useContext(LangContext);
  return { lang, setLang };
}

/** The translator for the current language. */
export function useT() {
  return useContext(LangContext).t;
}
