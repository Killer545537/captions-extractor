import {
    ActionButtons,
    AiToggle,
    Footer,
    Header,
    TranscriptArea,
    UrlInput,
} from '@/components/caption-extractor';
import {
    Card,
    CardContent,
    CardFooter,
    CardHeader,
} from '@/components/ui/card';
import { useCaptionExtractor } from '@/hooks/use-caption-extractor';
import './App.css';

/**
 * Main application component for the Caption Extractor.
 * Orchestrates the caption extraction workflow with optional AI cleaning.
 */
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
        refreshAiAvailability,
    } = useCaptionExtractor();

    // Derived state for UI logic
    const hasContent = transcript.length > 0 || url.length > 0;
    const isProcessing = isLoading || isCleaning;

    return (
        <div className='flex min-h-screen flex-col items-center justify-center px-4 py-12'>
            <Header onSettingsChange={refreshAiAvailability} />

            {/* Main extraction card */}
            <Card className='w-full max-w-4xl'>
                <CardHeader className='space-y-4 pb-4'>
                    <UrlInput
                        value={url}
                        onChange={setUrl}
                        onSubmit={extract}
                        isLoading={isLoading}
                        isProcessing={isProcessing}
                        useAi={useAi}
                    />
                    <AiToggle
                        checked={useAi}
                        onCheckedChange={setUseAi}
                        disabled={isProcessing}
                        isAvailable={isAiAvailable}
                    />
                </CardHeader>

                <CardContent>
                    <TranscriptArea
                        transcript={transcript}
                        error={error}
                        isLoading={isLoading}
                        isCleaning={isCleaning}
                        useAi={useAi}
                        wordCount={wordCount}
                        charCount={charCount}
                        wasAiCleaned={wasAiCleaned}
                    />
                </CardContent>

                <CardFooter className='pt-2'>
                    <ActionButtons
                        onCopy={copyToClipboard}
                        onClean={cleanWithAi}
                        onClear={clear}
                        hasTranscript={transcript.length > 0}
                        hasContent={hasContent}
                        isProcessing={isProcessing}
                        isCleaning={isCleaning}
                        isAiAvailable={isAiAvailable}
                        wasAiCleaned={wasAiCleaned}
                    />
                </CardFooter>
            </Card>

            <Footer isAiAvailable={isAiAvailable} />
        </div>
    );
};

export default App;
