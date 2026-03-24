import { createContext, useContext, useState, useEffect, ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import enTranslations from "./locales/en.json";
import zhTranslations from "./locales/zh.json";

type Translations = typeof enTranslations;
type Language = "en" | "zh";

const translations: Record<Language, Translations> = {
  en: enTranslations,
  zh: zhTranslations,
};

interface I18nContextType {
  language: Language;
  setLanguage: (lang: Language) => void;
  t: (key: string, params?: Record<string, string>) => string;
}

const I18nContext = createContext<I18nContextType | undefined>(undefined);

function getInitialLanguage(): Language {
  // Try to detect from browser
  const browserLang = navigator.language.toLowerCase();
  if (browserLang.startsWith("zh")) {
    return "zh";
  }
  
  return "en";
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [language, setLanguageState] = useState<Language>(getInitialLanguage);

  // Load language from backend settings on mount
  useEffect(() => {
    invoke<{ language: string }>("get_settings")
      .then((settings) => {
        if (settings.language === "en" || settings.language === "zh") {
          setLanguageState(settings.language);
        }
      })
      .catch(() => {
        // Ignore errors, use default language
      });
  }, []);

  const setLanguage = async (lang: Language) => {
    setLanguageState(lang);
    // Update backend settings
    try {
      await invoke("set_language", { lang });
    } catch (e) {
      console.error("Failed to set language:", e);
    }
  };

  const t = (key: string, params?: Record<string, string>): string => {
    const keys = key.split(".");
    let value: any = translations[language];
    
    for (const k of keys) {
      if (value && typeof value === "object" && k in value) {
        value = value[k];
      } else {
        return key; // Return key if translation not found
      }
    }
    
    if (typeof value !== "string") {
      return key;
    }
    
    // Replace placeholders like {{status}}
    if (params) {
      return value.replace(/\{\{(\w+)\}\}/g, (match, paramKey) => {
        return params[paramKey] || match;
      });
    }
    
    return value;
  };

  return (
    <I18nContext.Provider value={{ language, setLanguage, t }}>
      {children}
    </I18nContext.Provider>
  );
}

export function useI18n() {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error("useI18n must be used within I18nProvider");
  }
  return context;
}
