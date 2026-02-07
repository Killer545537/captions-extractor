import { Sparkles } from 'lucide-react';
import { formatNumber } from '@/lib/utils';

interface TranscriptStatsProps {
    /** Number of words in the transcript */
    wordCount: number;
    /** Number of characters in the transcript */
    charCount: number;
    /** Whether the transcript was cleaned by AI */
    wasAiCleaned: boolean;
}

/** Displays transcript statistics: word count, character count, and AI badge */
export function TranscriptStats({
    wordCount,
    charCount,
    wasAiCleaned,
}: TranscriptStatsProps) {
    // Don't render if there's no content
    if (wordCount === 0) return null;

    return (
        <div className='flex items-center gap-4 text-xs text-muted-foreground'>
            <span>{formatNumber(wordCount)} words</span>
            <span>{formatNumber(charCount)} characters</span>

            {/* AI Enhanced badge - shown when transcript was cleaned */}
            {wasAiCleaned && (
                <span className='flex items-center gap-1 rounded-full bg-primary/10 px-2 py-0.5 text-primary'>
                    <Sparkles className='h-3 w-3' />
                    AI Enhanced
                </span>
            )}
        </div>
    );
}
