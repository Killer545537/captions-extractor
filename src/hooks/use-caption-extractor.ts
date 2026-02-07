import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';

/** Result from extract_captions_with_options command */
interface PipelineResult {
    transcript: string;
    ai_cleaned: boolean;
}

export interface UseCaptionExtractorResult {
    url: string;
    setUrl: (url: string) => void;
    transcript: string;
    isLoading: boolean;
    isCleaning: boolean;
    error: string;
    useAi: boolean;
    setUseAi: (useAi: boolean) => void;
    isAiAvailable: boolean;
    wasAiCleaned: boolean;
    extract: () => Promise<void>;
    cleanWithAi: () => Promise<void>;
    clear: () => void;
    copyToClipboard: () => Promise<boolean>;
    wordCount: number;
    charCount: number;
}

const YOUTUBE_URL_REGEX =
    /^(https?:\/\/)?(www\.)?(youtube\.com\/(watch\?v=|shorts\/|embed\/)|youtu\.be\/)[\w-]+/i;

function isValidYouTubeUrl(url: string): boolean {
    return YOUTUBE_URL_REGEX.test(url.trim());
}

/** Check if Groq API key is configured */
async function checkAiAvailability(): Promise<boolean> {
    try {
        return await invoke<boolean>('is_ai_available');
    } catch {
        return false;
    }
}

/** Extract captions with optional AI cleaning */
async function extractCaptionsWithOptions(
    url: string,
    useAi: boolean,
): Promise<PipelineResult> {
    return invoke<PipelineResult>('extract_captions_with_options', {
        url,
        useAi,
    });
}

/** Clean existing transcript with AI */
async function cleanTranscriptWithAi(text: string): Promise<string> {
    return invoke<string>('clean_transcript_with_ai', { text });
}

export function useCaptionExtractor(): UseCaptionExtractorResult {
    const [url, setUrl] = useState('');
    const [transcript, setTranscript] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [isCleaning, setIsCleaning] = useState(false);
    const [error, setError] = useState('');
    const [useAi, setUseAi] = useState(false);
    const [isAiAvailable, setIsAiAvailable] = useState(false);
    const [wasAiCleaned, setWasAiCleaned] = useState(false);

    // Check AI availability on mount
    useEffect(() => {
        checkAiAvailability().then(setIsAiAvailable);
    }, []);

    const wordCount = transcript
        ? transcript.split(/\s+/).filter((word) => word.length > 0).length
        : 0;

    const charCount = transcript.length;

    const extract = useCallback(async () => {
        const trimmedUrl = url.trim();

        if (!trimmedUrl) {
            const msg = 'Please enter a YouTube URL.';
            setError(msg);
            toast.warning(msg);
            return;
        }

        if (!isValidYouTubeUrl(trimmedUrl)) {
            const msg =
                'Please enter a valid YouTube URL (e.g., youtube.com/watch?v=... or youtu.be/...)';
            setError(msg);
            toast.warning('Invalid YouTube URL');
            return;
        }

        setError('');
        setIsLoading(true);
        setWasAiCleaned(false);

        try {
            const result = await extractCaptionsWithOptions(
                trimmedUrl,
                useAi && isAiAvailable,
            );
            setTranscript(result.transcript);
            setWasAiCleaned(result.ai_cleaned);

            // Success toast with context
            const wordCount = result.transcript
                .split(/\s+/)
                .filter((w) => w.length > 0).length;
            toast.success('Captions extracted', {
                description: result.ai_cleaned
                    ? `${wordCount.toLocaleString()} words • AI enhanced`
                    : `${wordCount.toLocaleString()} words`,
            });
        } catch (err) {
            const errorMessage =
                typeof err === 'string'
                    ? err
                    : err instanceof Error
                      ? err.message
                      : 'Failed to extract captions. Please try again.';
            setError(errorMessage);
            toast.error('Extraction failed', {
                description: errorMessage,
            });
        } finally {
            setIsLoading(false);
        }
    }, [url, useAi, isAiAvailable]);

    const cleanWithAi = useCallback(async () => {
        if (!transcript || !isAiAvailable) return;

        setError('');
        setIsCleaning(true);

        try {
            const cleaned = await cleanTranscriptWithAi(transcript);
            setTranscript(cleaned);
            setWasAiCleaned(true);
            toast.success('Transcript cleaned with AI');
        } catch (err) {
            const errorMessage =
                typeof err === 'string'
                    ? err
                    : err instanceof Error
                      ? err.message
                      : 'Failed to clean transcript with AI.';
            setError(errorMessage);
            toast.error('AI cleaning failed', {
                description: errorMessage,
            });
        } finally {
            setIsCleaning(false);
        }
    }, [transcript, isAiAvailable]);

    const clear = useCallback(() => {
        setUrl('');
        setTranscript('');
        setError('');
        setWasAiCleaned(false);
        toast.info('Cleared');
    }, []);

    const copyToClipboard = useCallback(async (): Promise<boolean> => {
        if (!transcript) return false;

        try {
            await navigator.clipboard.writeText(transcript);
            toast.success('Copied to clipboard');
            return true;
        } catch {
            const msg = 'Failed to copy to clipboard.';
            setError(msg);
            toast.error(msg);
            return false;
        }
    }, [transcript]);

    return {
        url,
        setUrl,
        transcript,
        isLoading,
        isCleaning,
        error,
        useAi,
        setUseAi,
        isAiAvailable,
        wasAiCleaned,
        extract,
        cleanWithAi,
        clear,
        copyToClipboard,
        wordCount,
        charCount,
    };
}
