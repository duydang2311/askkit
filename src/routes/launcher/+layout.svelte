<script lang="ts">
    import { goto } from '$app/navigation';
    import { setLauncherContext } from '$lib/common/launcher.svelte';
    import { useCurrentAgent } from '$lib/common/queries';
    import { invoke } from '@tauri-apps/api/core';
    import { LogicalSize } from '@tauri-apps/api/dpi';
    import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
    import { onMount } from 'svelte';
    import { on } from 'svelte/events';
    import Bot from '~icons/lucide/bot';
    import type { LayoutProps } from './$types';
    import Greetings from './Greetings.svelte';
    import { page } from '$app/state';

    const { children }: LayoutProps = $props();
    const currentAgent = useCurrentAgent();
    const ctx = setLauncherContext({});

    let lastHeight = 0;

    $effect(() => {
        if (ctx.chatId) {
            localStorage.setItem('active_chat_id', ctx.chatId);
        } else {
            localStorage.removeItem('active_chat_id');
        }
    });

    onMount(() => {
        const off = on(window, 'keyup', async (e) => {
            if (e.key === 'Escape') {
                await invoke('destroy_launcher_window');
            }
        });
        return () => {
            localStorage.removeItem('active_chat_id');
            off();
        }
    });
</script>

<main
    {@attach (node) => {
        const observer = new ResizeObserver(() => {
            // avoid flickering due to sub-pixel rendering differences, especially when typing
            if (Math.abs(node.scrollHeight - lastHeight) <= 1) {
                return;
            }
            lastHeight = node.scrollHeight;
            getCurrentWebviewWindow().setSize(new LogicalSize(node.scrollWidth, node.scrollHeight));
        });
        observer.observe(node);
        return () => {
            observer.disconnect();
        };
    }}
>
    <div class="flex justify-between gap-4 px-6 py-2 border-b border-b-base-border">
        <Greetings />
        <p class="text-primary font-bold tracking-tight">askkit</p>
    </div>
    {@render children()}
    <div class="px-6 py-2 flex items-center gap-2 border-t border-t-base-border">
        <button
            id="agent"
            type="button"
            class="p-1 size-8 disabled:text-base-fg-muted"
            onclick={async () => {
                await goto(
                    page.url.pathname === '/launcher/settings'
                        ? '/launcher/chats'
                        : '/launcher/settings'
                );
            }}
        >
            <Bot class="size-full" />
        </button>
        {#if $currentAgent.data}
            <label
                for="agent"
                class="bg-base-dark dark:bg-base-light text-base-fg-muted text-xs p-1 leading-none content-center"
            >
                {$currentAgent.data.model}
            </label>
        {/if}
    </div>
</main>
