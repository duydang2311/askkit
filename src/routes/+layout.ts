import { QueryClient } from '@tanstack/svelte-query';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import type { LayoutLoad } from './$types';

// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const ssr = false;

export const load: LayoutLoad = async () => {
    const theme = await getCurrentWebviewWindow().theme();
    return { queryClient: new QueryClient(), theme };
};

