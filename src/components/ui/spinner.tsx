import { cn } from '@/lib/utils';

interface SpinnerProps {
    size?: 'sm' | 'md' | 'lg';
    className?: string;
}

const sizeClasses = {
    sm: 'h-4 w-4 border-2',
    md: 'h-6 w-6 border-2',
    lg: 'h-8 w-8 border-3',
};

export function Spinner({ size = 'md', className }: SpinnerProps) {
    return (
        <output aria-label='Loading' className='inline-flex'>
            <span
                className={cn(
                    'animate-spin rounded-full border-current border-t-transparent',
                    sizeClasses[size],
                    className,
                )}
            />
            <span className='sr-only'>Loading...</span>
        </output>
    );
}
