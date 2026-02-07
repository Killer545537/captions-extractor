import { Alert, AlertDescription } from '@/components/ui/alert';
import { Textarea } from '@/components/ui/textarea';
import { TranscriptStats } from './transcript-stats';

interface TranscriptAreaProps {
    /** The transcript text content */
    transcript: string;
    /** Error message to display, if any */
    error: string | null;
    /** Whether extraction is in progress */
    isLoading: boolean;
    /** Whether AI cleaning is in progress */
    isCleaning: boolean;
    /** Whether AI cleaning is enabled for extraction */
    useAi: boolean;
    /** Word count of the transcript */
    wordCount: number;
    /** Character count of the transcript */
    charCount: number;
    /** Whether the transcript was cleaned by AI */
    wasAiCleaned: boolean;
}

/** Displays the extracted transcript with stats and error handling */
export function TranscriptArea({
    transcript,
    error,
    isLoading,
    isCleaning,
    useAi,
    wordCount,
    charCount,
    wasAiCleaned,
}: TranscriptAreaProps) {
    // Dynamic placeholder based on current state
    const getPlaceholder = () => {
        if (isLoading) {
            return useAi
                ? 'Extracting and cleaning with AI...'
                : 'Extracting captions...';
        }
        if (isCleaning) {
            return 'Cleaning transcript with AI...';
        }
        return 'Transcript will appear here after extraction';
    };

    return (
        <div className='space-y-4'>
            {/* Error alert */}
            {error && (
                <Alert variant='destructive'>
                    <AlertDescription>{error}</AlertDescription>
                </Alert>
            )}

            {/* Transcript display */}
            <div className='space-y-2'>
                <div className='flex items-center justify-between'>
                    <label
                        htmlFor='transcript'
                        className='text-sm font-medium text-foreground'
                    >
                        Transcript
                    </label>
                    <TranscriptStats
                        wordCount={wordCount}
                        charCount={charCount}
                        wasAiCleaned={wasAiCleaned}
                    />
                </div>
                <Textarea
                    id='transcript'
                    readOnly
                    value={transcript}
                    placeholder={getPlaceholder()}
                    className='min-h-75 resize-y font-mono text-sm leading-relaxed'
                />
            </div>
        </div>
    );
}
