import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';

export interface UseSettingsResult {
    /** Whether AI features are available (API key configured) */
    isAiAvailable: boolean;
    /** Masked version of the current API key (for display) */
    maskedApiKey: string | null;
    /** Whether the API key is stored in app settings (vs environment) */
    isApiKeyStored: boolean;
    /** Whether settings are being loaded */
    isLoading: boolean;
    /** Whether an API key operation is in progress */
    isSaving: boolean;
    /** Save a new API key */
    saveApiKey: (apiKey: string) => Promise<boolean>;
    /** Remove the stored API key */
    removeApiKey: () => Promise<boolean>;
    /** Refresh settings from backend */
    refresh: () => Promise<void>;
}

/** Hook for managing application settings (API keys, etc.) */
export function useSettings(): UseSettingsResult {
    const [isAiAvailable, setIsAiAvailable] = useState(false);
    const [maskedApiKey, setMaskedApiKey] = useState<string | null>(null);
    const [isApiKeyStored, setIsApiKeyStored] = useState(false);
    const [isLoading, setIsLoading] = useState(true);
    const [isSaving, setIsSaving] = useState(false);

    /** Fetch current settings from backend */
    const refresh = useCallback(async () => {
        try {
            const [available, masked, stored] = await Promise.all([
                invoke<boolean>('is_ai_available'),
                invoke<string | null>('get_api_key_masked'),
                invoke<boolean>('is_api_key_stored'),
            ]);

            setIsAiAvailable(available);
            setMaskedApiKey(masked);
            setIsApiKeyStored(stored);
        } catch (error) {
            console.error('Failed to load settings:', error);
        } finally {
            setIsLoading(false);
        }
    }, []);

    // Load settings on mount
    useEffect(() => {
        refresh();
    }, [refresh]);

    /** Save a new API key */
    const saveApiKey = useCallback(
        async (apiKey: string): Promise<boolean> => {
            if (!apiKey.trim()) {
                toast.warning('API key cannot be empty');
                return false;
            }

            setIsSaving(true);
            try {
                await invoke('save_api_key', { apiKey: apiKey.trim() });
                await refresh();
                toast.success('API key saved');
                return true;
            } catch (error) {
                const message =
                    error instanceof Error ? error.message : String(error);
                toast.error('Failed to save API key', { description: message });
                return false;
            } finally {
                setIsSaving(false);
            }
        },
        [refresh],
    );

    /** Remove the stored API key */
    const removeApiKey = useCallback(async (): Promise<boolean> => {
        setIsSaving(true);
        try {
            await invoke('remove_api_key');
            await refresh();
            toast.success('API key removed');
            return true;
        } catch (error) {
            const message =
                error instanceof Error ? error.message : String(error);
            toast.error('Failed to remove API key', { description: message });
            return false;
        } finally {
            setIsSaving(false);
        }
    }, [refresh]);

    return {
        isAiAvailable,
        maskedApiKey,
        isApiKeyStored,
        isLoading,
        isSaving,
        saveApiKey,
        removeApiKey,
        refresh,
    };
}
