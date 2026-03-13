'use client';

import { createContext, useContext, useCallback, ReactNode } from 'react';
import { useConfig } from './ConfigContext';
import { toast } from 'sonner';

interface ImportDialogContextType {
  openImportDialog: (filePath?: string | null) => void;
}

const ImportDialogContext = createContext<ImportDialogContextType | null>(null);

export const useImportDialog = () => {
  const ctx = useContext(ImportDialogContext);
  if (!ctx) throw new Error('useImportDialog must be used within ImportDialogProvider');
  return ctx;
};

interface ImportDialogProviderProps {
  children: ReactNode;
  onOpen: (filePath?: string | null) => void;
}

export function ImportDialogProvider({ children, onOpen }: ImportDialogProviderProps) {
  const config = useConfig();

  const openImportDialog = useCallback((filePath?: string | null) => {
    // Gate: Check beta feature flag before opening dialog
    // Handle SSR case where config is null
    if (!config?.betaFeatures.importAndRetranscribe) {
      toast.error('Beta feature disabled', {
        description: 'Enable "Import Audio & Retranscribe" in Settings > Beta to use this feature.'
      });
      return;
    }

    onOpen(filePath);
  }, [onOpen, config?.betaFeatures]);

  return (
    <ImportDialogContext.Provider value={{ openImportDialog }}>
      {children}
    </ImportDialogContext.Provider>
  );
}
