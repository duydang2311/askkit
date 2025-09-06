import type { EventCallback, EventName, Options } from '@tauri-apps/api/event';
import { onMount } from 'svelte';
import { listen as __listen } from '@tauri-apps/api/event';

export const onEvent = <T>(
    event: EventName,
    handler: EventCallback<T>,
    options?: Options
) => {
    onMount(() => {
        const unlisten = __listen(event, handler, options);
        return () => {
            unlisten.then((fn) => fn());
        };
    });
};
