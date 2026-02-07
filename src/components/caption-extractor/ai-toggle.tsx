import { Sparkles } from 'lucide-react';
import { Switch } from '@/components/ui/switch';

interface AiToggleProps {
    /** Whether AI cleaning is enabled */
    checked: boolean;
    /** Callback when toggle state changes */
    onCheckedChange: (checked: boolean) => void;
    /** Whether the toggle should be disabled */
    disabled: boolean;
    /** Whether the AI feature is available (API key configured) */
    isAvailable: boolean;
}

/** Toggle switch for enabling/disabling AI cleaning during extraction */
export function AiToggle({
    checked,
    onCheckedChange,
    disabled,
    isAvailable,
}: AiToggleProps) {
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

            {/* Help text when API key is not configured */}
            {!isAvailable && (
                <span className='text-xs text-muted-foreground'>
                    (Set GROQ_API_KEY to enable)
                </span>
            )}
        </div>
    );
}
