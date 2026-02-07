import { ClipboardCopy } from 'lucide-react';
import { Button } from '@/components/ui/button';

interface CopyButtonProps {
    /** Async function that copies content, returns success status */
    onCopy: () => Promise<boolean>;
    /** Whether the button should be disabled */
    disabled: boolean;
}

/** Button that copies content to clipboard (toast handles feedback) */
export function CopyButton({ onCopy, disabled }: CopyButtonProps) {
    return (
        <Button
            variant='secondary'
            onClick={onCopy}
            disabled={disabled}
            className='flex-1 gap-2'
        >
            <ClipboardCopy className='h-4 w-4' />
            Copy to Clipboard
        </Button>
    );
}
