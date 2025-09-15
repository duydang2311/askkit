<script lang="ts">
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import { button } from '$lib/common/styles';
    import { RestartOutline } from '$lib/components/icons';
    import { invoke } from '@tauri-apps/api/core';
    import { LogicalSize } from '@tauri-apps/api/dpi';
    import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
    import { onMount } from 'svelte';
    import { on } from 'svelte/events';
    import Bot from '~icons/lucide/bot';
    import type { LayoutProps } from './$types';
    import Greetings from './Greetings.svelte';
    import { persisted } from './persisted.svelte';
    import SelectAgent from './SelectAgent.svelte';

    const { children }: LayoutProps = $props();

    let lastHeight = 0;

    onMount(() => {
        const off = on(window, 'keyup', async (e) => {
            if (e.key === 'Escape') {
                await invoke('destroy_launcher_window');
            }
        });
        return () => {
            localStorage.removeItem('active_chat_id');
            off();
        };
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
    <div class="border-b-base-border flex justify-between gap-4 border-b px-6 py-2">
        <Greetings />
        <p class="text-primary font-bold tracking-tight">askkit</p>
    </div>
    {@render children()}
    <div class="border-t-base-border flex gap-2 border-t px-6 py-2 text-xs">
        <button
            id="agent"
            type="button"
            class={button({
                variant: page.url.pathname === '/launcher/settings' ? 'primary' : 'base',
                filled: true,
            })}
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
        <SelectAgent />
        {#if persisted.chatId != null}
            <button
                type="button"
                class="{button({ variant: 'base', filled: true })} flex items-center gap-2"
                onclick={async () => {
                    persisted.chatId = null;
                    persisted.messages = null;
                    await goto('/launcher/chats');
                }}
            >
                <RestartOutline />
                New chat
            </button>
        {/if}
    </div>
</main>
