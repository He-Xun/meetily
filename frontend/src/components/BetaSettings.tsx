"use client"

import { Switch } from "./ui/switch"
import { FlaskConical, AlertCircle } from "lucide-react"
import { useConfig } from "@/contexts/ConfigContext"
import { BetaFeatureKey, getBetaFeatureNames, getBetaFeatureDescriptions } from "@/types/betaFeatures"
import { useI18n } from "@/i18n"

export function BetaSettings() {
  const config = useConfig();
  const { t } = useI18n();

  // Handle SSR case where config is null
  if (!config) {
    return null;
  }

  const { betaFeatures, toggleBetaFeature } = config;

  // Define feature order for display (allows custom ordering)
  const featureOrder: BetaFeatureKey[] = ['importAndRetranscribe'];

  // Get localized feature names and descriptions
  const featureNames = getBetaFeatureNames(t);
  const featureDescriptions = getBetaFeatureDescriptions(t);

  return (
    <div className="space-y-6">
      {/* Yellow Warning Banner */}
      <div className="flex items-start gap-3 p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
        <AlertCircle className="h-5 w-5 text-yellow-600 flex-shrink-0 mt-0.5" />
        <div className="text-sm text-yellow-800">
          <p className="font-medium">{t('settings.beta')}</p>
          <p className="mt-1">
            {t('settings.betaFeatureDescription')}
          </p>
        </div>
      </div>

      {/* Dynamic Feature Toggles - Automatically renders all features */}
      {featureOrder.map((featureKey) => (
        <div
          key={featureKey}
          className="bg-white rounded-lg border border-gray-200 p-6 shadow-sm"
        >
          <div className="flex items-center justify-between">
            <div className="flex-1">
              <div className="flex items-center gap-2 mb-2">
                <FlaskConical className="h-5 w-5 text-gray-600" />
                <h3 className="text-lg font-semibold text-gray-900">
                  {featureNames[featureKey]}
                </h3>
                <span className="px-2 py-0.5 text-xs font-medium bg-yellow-100 text-yellow-800 rounded-full">
                  {t('common.beta')}
                </span>
              </div>
              <p className="text-sm text-gray-600">
                {featureDescriptions[featureKey]}
              </p>
            </div>

            <div className="ml-6">
              <Switch
                checked={betaFeatures[featureKey]}
                onCheckedChange={(checked) => toggleBetaFeature(featureKey, checked)}
              />
            </div>
          </div>
        </div>
      ))}

      {/* Info Box */}
      <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
        <p className="text-sm text-blue-800">
          <strong>{t('common.note')}:</strong> {t('settings.betaFeatureInfoBox')}
        </p>
      </div>
    </div>
  );
}
