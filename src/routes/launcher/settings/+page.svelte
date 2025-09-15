<script lang="ts">
    import { useAgents, useCurrentAgent } from '$lib/common/queries';
    import { useRuntime } from '$lib/common/runtime';
    import { createPasswordInput, createSelect } from '$lib/components/builders.svelte';
    import { Eye, EyeSlash } from '$lib/components/icons';
    import { attempt } from '@duydang2311/attempt';
    import { createQuery } from '@tanstack/svelte-query';
    import { invoke } from '@tauri-apps/api/core';
    import { ListCollection } from '@zag-js/collection';
    import { portal } from '@zag-js/svelte';
    import { toStore } from 'svelte/store';

    const { queryClient } = useRuntime();

    const agents = useAgents();
    const currentAgent = useCurrentAgent();
    const agentConfig = createQuery(
        toStore(() => ({
            enabled: $currentAgent.data != null,
            queryKey: ['agent-config', { id: $currentAgent.data?.id }],
            queryFn: () =>
                invoke<{ agent_id: string; api_key: string } | null>('get_agent_config', {
                    id: $currentAgent.data!.id,
                }),
        }))
    );

    const id = $props.id();
    const select = createSelect({
        id: id + '-select',
        get collection() {
            return new ListCollection({
                items: $agents.data ?? [],
                itemToString: (item) => item.model,
                itemToValue: (item) => item.id,
            });
        },
        get value() {
            return $currentAgent.data ? [$currentAgent.data.id] : undefined;
        },
        onSelect: async (details) => {
            showApiKey = false;
            await invoke('update_current_agent', { agentId: details.value });
            await queryClient.invalidateQueries({ queryKey: ['current-agent'] });
        },
    });

    let showApiKey = $state.raw(false);
    let decryptedApiKey = $state.raw<string | null>(null);
    const passwordInput = createPasswordInput({
        id: id + '-password-input',
        get visible() {
            return showApiKey;
        },
        onVisibilityChange: async (details) => {
            let decryptedStr: string | null = null;
            if (details.visible) {
                const ciphertext = $agentConfig.data?.api_key;
                if (ciphertext) {
                    const decrypted = await attempt.async(() =>
                        invoke<string>('decrypt_agent_ciphertext', {
                            ciphertext,
                        })
                    )(console.error);
                    if (decrypted.ok) {
                        decryptedStr = decrypted.data;
                    }
                }
            }
            decryptedApiKey = decryptedStr;
            showApiKey = details.visible;
        },
    });
</script>

<div class="px-6 py-4">
    <div class="grid grid-cols-2 gap-4">
        <div>
            <h2 class="mb-2 text-base-fg-muted font-medium">Agent</h2>
            <div {...select.getRootProps()}>
                <div {...select.getControlProps()}>
                    <button
                        {...select.getTriggerProps()}
                        class="px-2 py-1 min-w-64 w-full text-left bg-base-light dark:bg-base-dark border border-base-border
                                hover:bg-base-hover data-[state=open]:bg-base-dark
                                focus-visible:outline-none focus-visible:ring focus-visible:ring-offset-2 focus-visible:ring-offset-base focus-visible:ring-base-border"
                    >
                        <label {...select.getLabelProps()} class="block c-label"> Model </label>
                        <span>
                            {select.valueAsString || 'Select agent'}
                        </span>
                    </button>
                </div>
                <div use:portal {...select.getPositionerProps()}>
                    <ul
                        {...select.getContentProps()}
                        class="w-(--reference-width) bg-base-light dark:bg-base-dark min-w-max border border-base-border p-1 focus:outline-none"
                    >
                        {#each $agents.data ?? [] as item (item.id)}
                            <li
                                {...select.getItemProps({ item })}
                                class="px-2 py-1 data-[highlighted]:bg-base-hover"
                            >
                                <span {...select.getItemTextProps({ item })}>{item.model}</span>
                                <span {...select.getItemIndicatorProps({ item })}>✓</span>
                            </li>
                        {/each}
                    </ul>
                </div>
            </div>
            <div class="mt-4 flex flex-col gap-2">
                {#if select.hasSelectedItems}
                    {@const item = select.selectedItems[0]}
                    <!-- why tf do i even need to null guard this but it happens -->
                    {#if item}
                        <div>
                            <p class="c-label">ID</p>
                            <p>{item.id}</p>
                        </div>
                        <div>
                            <p class="c-label">Provider</p>
                            <p>{item.provider}</p>
                        </div>
                    {/if}
                {/if}
            </div>
        </div>
        {#if select.hasSelectedItems}
            {@const item = select.selectedItems[0]}
            {#if item}
                <div>
                    <h2 class="mb-2 text-base-fg-muted font-medium">Parameters</h2>
                    {#if item.provider === 'gemini'}
                        <div
                            {...passwordInput.getRootProps()}
                            class={[
                                'border border-base-border bg-base-light dark:bg-base-dark px-2 py-1',
                                'focus-within:outline-none focus-within:ring focus-within:ring-offset-2 focus-within:ring-offset-base focus-within:ring-base-border',
                            ]}
                        >
                            <label {...passwordInput.getLabelProps()} class="block c-label">
                                API key
                            </label>
                            <div {...passwordInput.getControlProps()} class="flex gap-2">
                                <input
                                    {...passwordInput.getInputProps()}
                                    placeholder="Enter Gemini API key"
                                    value={showApiKey
                                        ? decryptedApiKey
                                        : ($agentConfig.data?.api_key ?? undefined)}
                                    class="w-full focus:outline-none placeholder:text-base-fg-muted"
                                    onblur={async (e) => {
                                        if (
                                            e.currentTarget.value ===
                                            ($agentConfig.data?.api_key ?? '')
                                        ) {
                                            return;
                                        }
                                        await invoke('upsert_agent_config', {
                                            id: item.id,
                                            upsert: {
                                                api_key: e.currentTarget.value,
                                            },
                                        });
                                        await queryClient.invalidateQueries({
                                            queryKey: ['agent-config', { id: item.id }],
                                        });
                                    }}
                                />
                                <button {...passwordInput.getVisibilityTriggerProps()}>
                                    <span {...passwordInput.getIndicatorProps()}>
                                        {#if passwordInput.visible}
                                            <Eye />
                                        {:else}
                                            <EyeSlash />
                                        {/if}
                                    </span>
                                </button>
                            </div>
                        </div>
                    {/if}
                </div>
            {/if}
        {/if}
    </div>
</div>
