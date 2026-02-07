import { Sparkles } from 'lucide-react';
import { Kbd } from '@/components/ui/kbd';

interface FooterProps {
    /** Whether AI features are available */
    isAiAvailable: boolean;
}

/** App footer with keyboard shortcuts hint and AI attribution */
export function Footer({ isAiAvailable }: FooterProps) {
    return (
        <footer className='mt-8 text-center text-xs text-muted-foreground'>
            {/* Keyboard shortcut hint */}
            <p>
                Tip: Press <Kbd>Enter</Kbd> to extract captions quickly
            </p>

            {/* AI attribution - shown when API key is configured */}
            {isAiAvailable && (
                <p className='mt-2 flex items-center justify-center gap-1'>
                    <Sparkles className='h-3 w-3' />
                    AI cleaning powered by Groq
                </p>
            )}
        </footer>
    );
}
