// Client-side i18n configuration for Tauri desktop app
// Supports UN official languages: English, Chinese, French, Russian, Spanish, Arabic

export const locales = ['en', 'zh', 'fr', 'ru', 'es', 'ar'] as const;
export type Locale = (typeof locales)[number];

export const localeNames: Record<Locale, string> = {
  en: 'English',
  zh: '中文',
  fr: 'Français',
  ru: 'Русский',
  es: 'Español',
  ar: 'العربية',
};

export const localeDirections: Record<Locale, 'ltr' | 'rtl'> = {
  en: 'ltr',
  zh: 'ltr',
  fr: 'ltr',
  ru: 'ltr',
  es: 'ltr',
  ar: 'rtl', // Arabic is right-to-left
};

// Get the default locale from localStorage or return 'en'
export function getDefaultLocale(): Locale {
  if (typeof window !== 'undefined') {
    const saved = localStorage.getItem('interfaceLanguage');
    if (saved && locales.includes(saved as Locale)) {
      return saved as Locale;
    }
  }
  return 'en';
}