import { Loader2, Sparkles, Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { CopyButton } from './copy-button';

interface ActionButtonsProps {
    /** Async function to copy transcript to clipboard */
    onCopy: () => Promise<boolean>;
    /** Callback to clean transcript with AI */
    onClean: () => void;
    /** Callback to clear all content */
    onClear: () => void;
    /** Whether there's a transcript to copy */
    hasTranscript: boolean;
    /** Whether there's any content (URL or transcript) */
    hasContent: boolean;
    /** Whether any processing is happening */
    isProcessing: boolean;
    /** Whether AI cleaning is in progress */
    isCleaning: boolean;
    /** Whether AI features are available */
    isAiAvailable: boolean;
    /** Whether the transcript was already cleaned by AI */
    wasAiCleaned: boolean;
}

/** Action buttons for copy, AI clean, and clear operations */
export function ActionButtons({
    onCopy,
    onClean,
    onClear,
    hasTranscript,
    hasContent,
    isProcessing,
    isCleaning,
    isAiAvailable,
    wasAiCleaned,
}: ActionButtonsProps) {
    // Show AI clean button when: AI is available, transcript exists, and wasn't already cleaned
    const showAiCleanButton = isAiAvailable && hasTranscript && !wasAiCleaned;

    return (
        <div className='flex flex-wrap gap-2'>
            {/* Copy to clipboard */}
            <CopyButton
                onCopy={onCopy}
                disabled={!hasTranscript || isProcessing}
            />

            {/* AI Clean - only shown when applicable */}
            {showAiCleanButton && (
                <Button
                    variant='outline'
                    onClick={onClean}
                    disabled={isProcessing}
                    className='gap-2'
                >
                    {isCleaning ? (
                        <>
                            <Loader2 className='h-4 w-4 animate-spin' />
                            Cleaning...
                        </>
                    ) : (
                        <>
                            <Sparkles className='h-4 w-4' />
                            Clean with AI
                        </>
                    )}
                </Button>
            )}

            {/* Clear all content */}
            <Button
                variant='outline'
                onClick={onClear}
                disabled={!hasContent || isProcessing}
                className='gap-2'
            >
                <Trash2 className='h-4 w-4' />
                Clear
            </Button>
        </div>
    );
}
