import {
    Check,
    ClipboardCopy,
    Loader2,
    Sparkles,
    Trash2,
    Youtube,
} from 'lucide-react';
import { useCallback, useState } from 'react';
import { ModeToggle } from '@/components/mode-toggle';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import {
    Card,
    CardContent,
    CardFooter,
    CardHeader,
} from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { useCaptionExtractor } from '@/hooks/use-caption-extractor';
import { formatNumber } from '@/lib/utils';
import './App.css';

function CopyButton({
    onCopy,
    disabled,
}: {
    onCopy: () => Promise<boolean>;
    disabled: boolean;
}) {
    const [copied, setCopied] = useState(false);

    const handleCopy = useCallback(async () => {
        const success = await onCopy();
        if (success) {
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    }, [onCopy]);

    return (
        <Button
            variant='secondary'
            onClick={handleCopy}
            disabled={disabled}
            className='flex-1 gap-2'
        >
            {copied ? (
                <>
                    <Check className='h-4 w-4' />
                    Copied!
                </>
            ) : (
                <>
                    <ClipboardCopy className='h-4 w-4' />
                    Copy to Clipboard
                </>
            )}
        </Button>
    );
}

function TranscriptStats({
    wordCount,
    charCount,
    wasAiCleaned,
}: {
    wordCount: number;
    charCount: number;
    wasAiCleaned: boolean;
}) {
    if (wordCount === 0) return null;

    return (
        <div className='flex items-center gap-4 text-xs text-muted-foreground'>
            <span>{formatNumber(wordCount)} words</span>
            <span>{formatNumber(charCount)} characters</span>
            {wasAiCleaned && (
                <span className='flex items-center gap-1 rounded-full bg-primary/10 px-2 py-0.5 text-primary'>
                    <Sparkles className='h-3 w-3' />
                    AI Enhanced
                </span>
            )}
        </div>
    );
}

function AiToggle({
    checked,
    onCheckedChange,
    disabled,
    isAvailable,
}: {
    checked: boolean;
    onCheckedChange: (checked: boolean) => void;
    disabled: boolean;
    isAvailable: boolean;
}) {
    return (
        <div className='flex items-center gap-3'>
            <div className='flex items-center gap-2'>
                <Switch
                    id='ai-toggle'
                    checked={checked}
                    onCheckedChange={onCheckedChange}
                    disabled={disabled || !isAvailable}
                    aria-label='Enable AI cleaning'
                />
                <label
                    htmlFor='ai-toggle'
                    className={`flex cursor-pointer items-center gap-1.5 text-sm font-medium ${
                        !isAvailable
                            ? 'cursor-not-allowed text-muted-foreground'
                            : ''
                    }`}
                >
                    <Sparkles className='h-4 w-4' />
                    AI Clean
                </label>
            </div>
            {!isAvailable && (
                <span className='text-xs text-muted-foreground'>
                    (Set GROQ_API_KEY to enable)
                </span>
            )}
        </div>
    );
}

const App = () => {
    const {
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
    } = useCaptionExtractor();

    const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Enter' && !isLoading && !isCleaning) {
            extract();
        }
    };

    const hasContent = transcript.length > 0 || url.length > 0;
    const isProcessing = isLoading || isCleaning;

    return (
        <div className='flex min-h-screen flex-col items-center justify-center px-4 py-12'>
            {/* Header */}
            <header className='mb-8 flex w-full max-w-4xl items-center justify-between'>
                <div className='flex items-center gap-3'>
                    <div className='flex h-10 w-10 items-center justify-center rounded-lg bg-primary'>
                        <Youtube className='h-5 w-5 text-primary-foreground' />
                    </div>
                    <div>
                        <h1 className='text-xl font-semibold tracking-tight text-foreground'>
                            Caption Extractor
                        </h1>
                        <p className='text-sm text-muted-foreground'>
                            Extract captions from YouTube videos
                        </p>
                    </div>
                </div>
                <ModeToggle />
            </header>

            {/* Main Card */}
            <Card className='w-full max-w-4xl'>
                <CardHeader className='space-y-4 pb-4'>
                    {/* URL Input Row */}
                    <div className='flex gap-2'>
                        <Input
                            placeholder='Paste a YouTube URL (e.g., youtube.com/watch?v=...)'
                            value={url}
                            onChange={(e) => setUrl(e.target.value)}
                            onKeyDown={handleKeyDown}
                            disabled={isProcessing}
                            className='flex-1'
                            aria-label='YouTube URL'
                        />
                        <Button
                            onClick={extract}
                            disabled={isProcessing || !url.trim()}
                            className='min-w-[120px] gap-2'
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

                    {/* AI Toggle Row */}
                    <AiToggle
                        checked={useAi}
                        onCheckedChange={setUseAi}
                        disabled={isProcessing}
                        isAvailable={isAiAvailable}
                    />
                </CardHeader>

                <CardContent className='space-y-4'>
                    {/* Error Alert */}
                    {error && (
                        <Alert variant='destructive'>
                            <AlertDescription>{error}</AlertDescription>
                        </Alert>
                    )}

                    {/* Transcript Area */}
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
                            placeholder={
                                isLoading
                                    ? useAi
                                        ? 'Extracting and cleaning with AI...'
                                        : 'Extracting captions...'
                                    : isCleaning
                                      ? 'Cleaning transcript with AI...'
                                      : 'Transcript will appear here after extraction'
                            }
                            className='min-h-[300px] resize-y font-mono text-sm leading-relaxed'
                        />
                    </div>
                </CardContent>

                <CardFooter className='flex flex-wrap gap-2 pt-2'>
                    <CopyButton
                        onCopy={copyToClipboard}
                        disabled={!transcript || isProcessing}
                    />

                    {/* AI Clean button - shown when transcript exists but wasn't AI cleaned */}
                    {isAiAvailable && transcript && !wasAiCleaned && (
                        <Button
                            variant='outline'
                            onClick={cleanWithAi}
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

                    <Button
                        variant='outline'
                        onClick={clear}
                        disabled={!hasContent || isProcessing}
                        className='gap-2'
                    >
                        <Trash2 className='h-4 w-4' />
                        Clear
                    </Button>
                </CardFooter>
            </Card>

            {/* Footer */}
            <footer className='mt-8 text-center text-xs text-muted-foreground'>
                <p>
                    Tip: Press{' '}
                    <kbd className='rounded bg-muted px-1.5 py-0.5 font-mono'>
                        Enter
                    </kbd>{' '}
                    to extract captions quickly
                </p>
                {isAiAvailable && (
                    <p className='mt-2 flex items-center justify-center gap-1'>
                        <Sparkles className='h-3 w-3' />
                        AI cleaning powered by Groq
                    </p>
                )}
            </footer>
        </div>
    );
};

export default App;
