import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

/**
 * Regular expression to match valid YouTube URLs
 * Supports:
 * - youtube.com/watch?v=VIDEO_ID
 * - youtu.be/VIDEO_ID
 * - youtube.com/embed/VIDEO_ID
 * - youtube.com/v/VIDEO_ID
 * - youtube.com/shorts/VIDEO_ID
 * - m.youtube.com/watch?v=VIDEO_ID
 */
const YOUTUBE_URL_REGEX =
    /^(https?:\/\/)?(www\.|m\.)?(youtube\.com\/(watch\?v=|embed\/|v\/|shorts\/)|youtu\.be\/)[\w-]{11}(&.*)?$/i;

/**
 * Validates if a given string is a valid YouTube URL
 * @param url - The URL string to validate
 * @returns true if the URL is a valid YouTube URL, false otherwise
 */
export function isValidYouTubeUrl(url: string): boolean {
    if (!url || typeof url !== 'string') {
        return false;
    }
    return YOUTUBE_URL_REGEX.test(url.trim());
}

/**
 * Extracts the video ID from a YouTube URL
 * @param url - The YouTube URL
 * @returns The video ID or null if not found
 */
export function extractVideoId(url: string): string | null {
    if (!url) return null;

    const patterns = [
        /(?:youtube\.com\/watch\?v=|youtu\.be\/|youtube\.com\/embed\/|youtube\.com\/v\/|youtube\.com\/shorts\/)([^&\n?#]+)/i,
    ];

    for (const pattern of patterns) {
        const match = url.match(pattern);
        if (match?.[1]) {
            return match[1];
        }
    }

    return null;
}

/**
 * Counts words in a text string
 * @param text - The text to count words in
 * @returns The number of words
 */
export function countWords(text: string): number {
    if (!text || typeof text !== 'string') {
        return 0;
    }
    return text.trim().split(/\s+/).filter(Boolean).length;
}

/**
 * Formats a number with locale-aware separators
 * @param num - The number to format
 * @returns Formatted number string
 */
export function formatNumber(num: number): string {
    return num.toLocaleString();
}
