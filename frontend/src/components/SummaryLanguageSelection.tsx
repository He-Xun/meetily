'use client';

import React from 'react';
import { Globe } from 'lucide-react';
import { useConfig } from '@/contexts/ConfigContext';
import { useI18n } from '@/i18n';
import Analytics from '@/lib/analytics';

// UN official languages supported for summary generation
export const SUMMARY_LANGUAGES = [
  { code: 'en', name: 'English', nativeName: 'English' },
  { code: 'zh', name: 'Chinese', nativeName: '中文' },
  { code: 'fr', name: 'French', nativeName: 'Français' },
  { code: 'ru', name: 'Russian', nativeName: 'Русский' },
  { code: 'es', name: 'Spanish', nativeName: 'Español' },
  { code: 'ar', name: 'Arabic', nativeName: 'العربية' },
] as const;

export type SummaryLanguageCode = typeof SUMMARY_LANGUAGES[number]['code'];

interface SummaryLanguageSelectionProps {
  disabled?: boolean;
  showLabel?: boolean;
  className?: string;
}

export function SummaryLanguageSelection({
  disabled = false,
  showLabel = true,
  className = ''
}: SummaryLanguageSelectionProps) {
  const { summaryLanguage, setSummaryLanguage } = useConfig();
  const { t } = useI18n();

  const handleLanguageChange = async (languageCode: string) => {
    setSummaryLanguage(languageCode);
    
    const selectedLang = SUMMARY_LANGUAGES.find(lang => lang.code === languageCode);
    await Analytics.track('summary_language_selected', {
      language_code: languageCode,
      language_name: selectedLang?.name || 'Unknown'
    });
  };

  const selectedLanguageName = SUMMARY_LANGUAGES.find(
    lang => lang.code === summaryLanguage
  )?.nativeName || 'English';

  return (
    <div className={`space-y-2 ${className}`}>
      {showLabel && (
        <div className="flex items-center gap-2">
          <Globe className="h-4 w-4 text-gray-600" />
          <label className="text-sm font-medium text-gray-900">
            {t('summary.summaryLanguage')}
          </label>
        </div>
      )}
      <p className="text-xs text-gray-500">
        {t('summary.summaryLanguageDescription')}
      </p>
      <select
        value={summaryLanguage}
        onChange={(e) => handleLanguageChange(e.target.value)}
        disabled={disabled}
        className="w-full px-3 py-2 text-sm bg-white border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-50 disabled:text-gray-500"
      >
        {SUMMARY_LANGUAGES.map((language) => (
          <option key={language.code} value={language.code}>
            {language.nativeName} ({language.name})
          </option>
        ))}
      </select>
    </div>
  );
}

// Compact version for inline use
export function SummaryLanguageSelectionCompact({
  disabled = false,
  className = ''
}: SummaryLanguageSelectionProps) {
  const { summaryLanguage, setSummaryLanguage } = useConfig();

  const handleLanguageChange = async (languageCode: string) => {
    setSummaryLanguage(languageCode);
    
    const selectedLang = SUMMARY_LANGUAGES.find(lang => lang.code === languageCode);
    await Analytics.track('summary_language_selected', {
      language_code: languageCode,
      language_name: selectedLang?.name || 'Unknown'
    });
  };

  return (
    <select
      value={summaryLanguage}
      onChange={(e) => handleLanguageChange(e.target.value)}
      disabled={disabled}
      className={`px-2 py-1 text-xs bg-white border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-50 disabled:text-gray-500 ${className}`}
    >
      {SUMMARY_LANGUAGES.map((language) => (
        <option key={language.code} value={language.code}>
          {language.nativeName}
        </option>
      ))}
    </select>
  );
}