import { writable } from 'svelte/store';

export type Theme = 'light' | 'dark' | 'system';

export interface AppSettings {
	theme: Theme;
	fontSize: number; // For UI, not editor content
	// Add more settings as needed
}

const DEFAULT_SETTINGS: AppSettings = {
	theme: 'system',
	fontSize: 14
};

function createSettingsStore() {
	// Load from localStorage if available
	const saved = typeof localStorage !== 'undefined' ? localStorage.getItem('loki_settings') : null;
	const initial = saved ? JSON.parse(saved) : DEFAULT_SETTINGS;

	const { subscribe, set, update } = writable<AppSettings>(initial);

	return {
		subscribe,
		set: (value: AppSettings) => {
			if (typeof localStorage !== 'undefined') {
				localStorage.setItem('loki_settings', JSON.stringify(value));
			}
			set(value);
		},
		updateSetting: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
			update((settings) => {
				const newSettings = { ...settings, [key]: value };
				if (typeof localStorage !== 'undefined') {
					localStorage.setItem('loki_settings', JSON.stringify(newSettings));
				}
				return newSettings;
			});
		},
		reset: () => {
			if (typeof localStorage !== 'undefined') {
				localStorage.removeItem('loki_settings');
			}
			set(DEFAULT_SETTINGS);
		}
	};
}

export const settingsStore = createSettingsStore();
