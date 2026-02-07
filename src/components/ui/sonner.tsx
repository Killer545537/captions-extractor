import {
    CircleCheckIcon,
    InfoIcon,
    Loader2Icon,
    OctagonXIcon,
    TriangleAlertIcon,
} from 'lucide-react';
import { Toaster as Sonner, type ToasterProps } from 'sonner';
import { useTheme } from '@/components/theme-provider';

/** Themed toast container using sonner */
const Toaster = ({ ...props }: ToasterProps) => {
    const { theme } = useTheme();

    // Resolve system theme to actual theme for sonner
    const resolvedTheme =
        theme === 'system'
            ? window.matchMedia('(prefers-color-scheme: dark)').matches
                ? 'dark'
                : 'light'
            : theme;

    return (
        <Sonner
            theme={resolvedTheme as ToasterProps['theme']}
            className='toaster group'
            icons={{
                success: <CircleCheckIcon className='size-4' />,
                info: <InfoIcon className='size-4' />,
                warning: <TriangleAlertIcon className='size-4' />,
                error: <OctagonXIcon className='size-4' />,
                loading: <Loader2Icon className='size-4 animate-spin' />,
            }}
            style={
                {
                    '--normal-bg': 'var(--popover)',
                    '--normal-text': 'var(--popover-foreground)',
                    '--normal-border': 'var(--border)',
                    '--border-radius': 'var(--radius)',
                } as React.CSSProperties
            }
            toastOptions={{
                classNames: {
                    toast: 'cn-toast',
                },
            }}
            {...props}
        />
    );
};

export { Toaster };
