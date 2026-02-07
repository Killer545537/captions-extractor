import { Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';

interface UrlInputProps {
    /** Current URL value */
    value: string;
    /** Callback when URL changes */
    onChange: (value: string) => void;
    /** Callback when Enter key is pressed */
    onSubmit: () => void;
    /** Whether extraction is in progress */
    isLoading: boolean;
    /** Whether any processing is happening (disables input) */
    isProcessing: boolean;
    /** Whether AI cleaning is enabled (affects loading text) */
    useAi: boolean;
}

/** URL input field with extract button for YouTube video URLs */
export function UrlInput({
    value,
    onChange,
    onSubmit,
    isLoading,
    isProcessing,
    useAi,
}: UrlInputProps) {
    // Handle Enter key to submit
    const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Enter' && !isProcessing) {
            onSubmit();
        }
    };

    return (
        <div className='flex gap-2'>
            <Input
                placeholder='Paste a YouTube URL (e.g., youtube.com/watch?v=...)'
                value={value}
                onChange={(e) => onChange(e.target.value)}
                onKeyDown={handleKeyDown}
                disabled={isProcessing}
                className='flex-1'
                aria-label='YouTube URL'
            />
            <Button
                onClick={onSubmit}
                disabled={isProcessing || !value.trim()}
                className='min-w-30 gap-2'
            >
                {isLoading ? (
                    <>
                        <Loader2 className='h-4 w-4 animate-spin' />
                        {useAi ? 'Processing' : 'Extracting'}
                    </>
                ) : (
                    'Extract'
                )}
            </Button>
        </div>
    );
}
