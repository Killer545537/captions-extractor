import { cva, type VariantProps } from 'class-variance-authority';
import { AlertCircle, CheckCircle2, Info, XCircle } from 'lucide-react';
import type * as React from 'react';

import { cn } from '@/lib/utils';

const alertVariants = cva(
    'relative w-full rounded-lg border px-4 py-3 text-sm grid has-[>svg]:grid-cols-[auto_1fr] gap-x-3 gap-y-0.5 items-start [&>svg]:size-4 [&>svg]:translate-y-0.5',
    {
        variants: {
            variant: {
                default: 'bg-card text-foreground border-border',
                destructive:
                    'bg-destructive/10 text-destructive border-destructive/20 dark:border-destructive/30 [&>svg]:text-destructive',
                success:
                    'bg-emerald-50 text-emerald-900 border-emerald-200 dark:bg-emerald-950/50 dark:text-emerald-100 dark:border-emerald-800 [&>svg]:text-emerald-600 dark:[&>svg]:text-emerald-400',
                warning:
                    'bg-amber-50 text-amber-900 border-amber-200 dark:bg-amber-950/50 dark:text-amber-100 dark:border-amber-800 [&>svg]:text-amber-600 dark:[&>svg]:text-amber-400',
                info: 'bg-blue-50 text-blue-900 border-blue-200 dark:bg-blue-950/50 dark:text-blue-100 dark:border-blue-800 [&>svg]:text-blue-600 dark:[&>svg]:text-blue-400',
            },
        },
        defaultVariants: {
            variant: 'default',
        },
    },
);

const iconMap = {
    default: Info,
    destructive: XCircle,
    success: CheckCircle2,
    warning: AlertCircle,
    info: Info,
} as const;

interface AlertProps
    extends React.ComponentProps<'div'>,
        VariantProps<typeof alertVariants> {
    showIcon?: boolean;
}

function Alert({
    className,
    variant = 'default',
    showIcon = true,
    children,
    ...props
}: AlertProps) {
    const Icon = iconMap[variant ?? 'default'];

    return (
        <div
            data-slot='alert'
            role='alert'
            className={cn(alertVariants({ variant }), className)}
            {...props}
        >
            {showIcon && <Icon />}
            <div className='flex flex-col gap-1'>{children}</div>
        </div>
    );
}

function AlertTitle({ className, ...props }: React.ComponentProps<'h5'>) {
    return (
        <h5
            data-slot='alert-title'
            className={cn('font-medium leading-none tracking-tight', className)}
            {...props}
        />
    );
}

function AlertDescription({ className, ...props }: React.ComponentProps<'p'>) {
    return (
        <p
            data-slot='alert-description'
            className={cn(
                'text-sm opacity-90 [&_p]:leading-relaxed',
                className,
            )}
            {...props}
        />
    );
}

export { Alert, AlertTitle, AlertDescription };
