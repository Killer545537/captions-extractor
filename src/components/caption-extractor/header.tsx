import { Youtube } from 'lucide-react';
import { ModeToggle } from '@/components/mode-toggle';

/** App header with branding and theme toggle */
export function Header() {
    return (
        <header className='mb-8 flex w-full max-w-4xl items-center justify-between'>
            {/* App branding */}
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

            {/* Theme toggle */}
            <ModeToggle />
        </header>
    );
}
