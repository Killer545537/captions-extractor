import { Eye, EyeOff, Key, Loader2, Settings, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { useSettings } from '@/hooks/use-settings';

interface SettingsDialogProps {
    /** Callback when settings are changed (API key saved/removed) */
    onSettingsChange?: () => void;
}

/** Dialog for managing API key settings */
export function SettingsDialog({ onSettingsChange }: SettingsDialogProps) {
    const {
        isAiAvailable,
        maskedApiKey,
        isApiKeyStored,
        isSaving,
        saveApiKey,
        removeApiKey,
        refresh,
    } = useSettings();

    const [open, setOpen] = useState(false);
    const [apiKeyInput, setApiKeyInput] = useState('');
    const [showApiKey, setShowApiKey] = useState(false);

    /** Handle saving the API key */
    const handleSave = async () => {
        const success = await saveApiKey(apiKeyInput);
        if (success) {
            setApiKeyInput('');
            setShowApiKey(false);
            onSettingsChange?.();
        }
    };

    /** Handle removing the API key */
    const handleRemove = async () => {
        const success = await removeApiKey();
        if (success) {
            onSettingsChange?.();
        }
    };

    /** Refresh settings when dialog opens */
    const handleOpenChange = (isOpen: boolean) => {
        setOpen(isOpen);
        if (isOpen) {
            refresh();
            setApiKeyInput('');
            setShowApiKey(false);
        }
    };

    return (
        <Dialog open={open} onOpenChange={handleOpenChange}>
            <DialogTrigger
                render={
                    <Button variant='ghost' size='icon' title='Settings'>
                        <Settings className='h-5 w-5' />
                        <span className='sr-only'>Settings</span>
                    </Button>
                }
            />
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>Settings</DialogTitle>
                    <DialogDescription>
                        Configure your API keys for AI features.
                    </DialogDescription>
                </DialogHeader>

                <div className='space-y-4'>
                    {/* Current API Key Status */}
                    <div className='space-y-2'>
                        <label className='text-sm font-medium'>
                            Groq API Key
                        </label>

                        {isAiAvailable ? (
                            <div className='flex items-center gap-2'>
                                <div className='flex-1 rounded-md border bg-muted/50 px-3 py-2 font-mono text-sm'>
                                    {maskedApiKey || '••••••••'}
                                </div>
                                {isApiKeyStored && (
                                    <Button
                                        variant='outline'
                                        size='icon'
                                        onClick={handleRemove}
                                        disabled={isSaving}
                                        title='Remove API key'
                                    >
                                        {isSaving ? (
                                            <Loader2 className='h-4 w-4 animate-spin' />
                                        ) : (
                                            <Trash2 className='h-4 w-4' />
                                        )}
                                    </Button>
                                )}
                            </div>
                        ) : (
                            <p className='text-sm text-muted-foreground'>
                                No API key configured. Enter your key below to
                                enable AI features.
                            </p>
                        )}

                        {/* Source indicator */}
                        {isAiAvailable && (
                            <p className='text-xs text-muted-foreground'>
                                {isApiKeyStored
                                    ? 'Key stored in app settings'
                                    : 'Key loaded from environment variable'}
                            </p>
                        )}
                    </div>

                    {/* New API Key Input */}
                    <div className='space-y-2'>
                        <label
                            htmlFor='api-key-input'
                            className='text-sm font-medium'
                        >
                            {isAiAvailable ? 'Update API Key' : 'Enter API Key'}
                        </label>
                        <div className='flex gap-2'>
                            <div className='relative flex-1'>
                                <Key className='absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground' />
                                <Input
                                    id='api-key-input'
                                    type={showApiKey ? 'text' : 'password'}
                                    placeholder='gsk_...'
                                    value={apiKeyInput}
                                    onChange={(e) =>
                                        setApiKeyInput(e.target.value)
                                    }
                                    className='pl-9 pr-9 font-mono'
                                    disabled={isSaving}
                                />
                                <button
                                    type='button'
                                    onClick={() => setShowApiKey(!showApiKey)}
                                    className='absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground'
                                    title={
                                        showApiKey
                                            ? 'Hide API key'
                                            : 'Show API key'
                                    }
                                >
                                    {showApiKey ? (
                                        <EyeOff className='h-4 w-4' />
                                    ) : (
                                        <Eye className='h-4 w-4' />
                                    )}
                                </button>
                            </div>
                            <Button
                                onClick={handleSave}
                                disabled={!apiKeyInput.trim() || isSaving}
                            >
                                {isSaving ? (
                                    <Loader2 className='h-4 w-4 animate-spin' />
                                ) : (
                                    'Save'
                                )}
                            </Button>
                        </div>
                        <p className='text-xs text-muted-foreground'>
                            Get your API key from{' '}
                            <a
                                href='https://console.groq.com/keys'
                                target='_blank'
                                rel='noopener noreferrer'
                                className='text-primary underline underline-offset-2 hover:text-primary/80'
                            >
                                console.groq.com
                            </a>
                        </p>
                    </div>
                </div>

                <DialogFooter showCloseButton />
            </DialogContent>
        </Dialog>
    );
}
