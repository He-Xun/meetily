'use client';

import React, { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';
import { locales, Locale, localeNames, localeDirections } from './config';

interface I18nContextType {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: string, params?: Record<string, string | number>) => string;
  direction: 'ltr' | 'rtl';
  localeNames: Record<Locale, string>;
  messages: Record<string, unknown>;
}

const I18nContext = createContext<I18nContextType | undefined>(undefined);

// Import all messages
const messagesMap: Record<Locale, () => Promise<Record<string, unknown>>> = {
  en: () => import('./messages/en.json').then(m => m.default),
  zh: () => import('./messages/zh.json').then(m => m.default),
  fr: () => import('./messages/fr.json').then(m => m.default),
  ru: () => import('./messages/ru.json').then(m => m.default),
  es: () => import('./messages/es.json').then(m => m.default),
  ar: () => import('./messages/ar.json').then(m => m.default),
};

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>('en');
  const [messages, setMessages] = useState<Record<string, unknown>>({});

  // Load initial locale from localStorage
  useEffect(() => {
    const savedLocale = localStorage.getItem('interfaceLanguage') as Locale;
    if (savedLocale && locales.includes(savedLocale)) {
      setLocaleState(savedLocale);
    }
  }, []);

  // Load messages when locale changes
  useEffect(() => {
    const loadMessages = async () => {
      try {
        const msgs = await messagesMap[locale]();
        setMessages(msgs);
      } catch (error) {
        console.error(`Failed to load messages for locale ${locale}:`, error);
        // Fallback to English if loading fails
        const fallbackMsgs = await messagesMap['en']();
        setMessages(fallbackMsgs);
      }
    };
    
    loadMessages();
  }, [locale]);

  // Set locale and persist
  const setLocale = useCallback((newLocale: Locale) => {
    setLocaleState(newLocale);
    localStorage.setItem('interfaceLanguage', newLocale);
    document.documentElement.lang = newLocale;
    document.documentElement.dir = localeDirections[newLocale];
  }, []);

  // Translation function with nested key support and parameter interpolation
  const t = useCallback((key: string, params?: Record<string, string | number>): string => {
    const keys = key.split('.');
    let value: unknown = messages;
    
    for (const k of keys) {
      if (value && typeof value === 'object' && k in value) {
        value = (value as Record<string, unknown>)[k];
      } else {
        return key; // Return key if not found
      }
    }
    
    if (typeof value === 'string') {
      // Replace parameters like {name} with actual values
      if (params) {
        return value.replace(/\{(\w+)\}/g, (_, paramKey) => 
          String(params[paramKey] ?? `{${paramKey}}`)
        );
      }
      return value;
    }
    
    return key;
  }, [messages]);

  const direction = localeDirections[locale];

  return (
    <I18nContext.Provider value={{ locale, setLocale, t, direction, localeNames, messages }}>
      {children}
    </I18nContext.Provider>
  );
}

export function useI18n() {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error('useI18n must be used within an I18nProvider');
  }
  return context;
}

export { locales, type Locale, localeNames, localeDirections };