import { writable } from 'svelte/store';

export const debugLogs = writable<string[]>([]);

export function addDebugLog(message: string) {
    const timestamp = new Date().toLocaleTimeString();
    debugLogs.update(logs => [`[${timestamp}] ${message}`, ...logs].slice(0, 100));
}

export function clearDebugLogs() {
    debugLogs.set([]);
}
