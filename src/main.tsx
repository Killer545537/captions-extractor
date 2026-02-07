import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { ThemeProvider } from './components/theme-provider';
import { Toaster } from './components/ui/sonner';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <React.StrictMode>
        <ThemeProvider>
            <App />
            {/* Toast notifications - top center for visibility */}
            <Toaster position='top-center' richColors closeButton />
        </ThemeProvider>
    </React.StrictMode>,
);
