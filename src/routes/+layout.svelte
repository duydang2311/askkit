<script lang="ts">
    import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
    import '../app.css';
    import '@fontsource-variable/inter';
    import { QueryClientProvider } from '@tanstack/svelte-query';
    import { setRuntime } from '$lib/common/runtime';

    const { data, children } = $props();
    setRuntime({ queryClient: data.queryClient });
</script>

<svelte:document
    {@attach (node) => {
        const { theme } = data;
        node.documentElement.setAttribute('data-theme', theme ?? 'light');

        const unlisten = getCurrentWebviewWindow().onThemeChanged((e) => {
            node.documentElement.setAttribute('data-theme', e.payload ?? 'light');
        });
        return () => {
            unlisten.then((a) => a());
        };
    }}
/>

<QueryClientProvider client={data.queryClient}>
    {@render children()}
</QueryClientProvider>
