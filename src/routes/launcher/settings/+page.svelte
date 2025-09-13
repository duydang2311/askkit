<script lang="ts">
    import { useCurrentAgent } from '$lib/common/queries';
    import { useRuntime } from '$lib/common/runtime';
    import Select from '$lib/components/Select.svelte';
    import { createQuery } from '@tanstack/svelte-query';
    import { invoke } from '@tauri-apps/api/core';
    import { ListCollection } from '@zag-js/collection';
    import { portal } from '@zag-js/svelte';
    import { toStore } from 'svelte/store';

    const { queryClient } = useRuntime();
    const agents = createQuery({
        queryKey: ['agents'],
        queryFn: () =>
            invoke<
                {
                    id: string;
                    provider: string;
                    model: string;
                }[]
            >('get_agents'),
    });
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
    let select = $state.raw<Select<{ id: string; provider: string; model: string }>>();
    const selectApi = $derived(select?.api());
</script>

<div class="px-6 py-4">
    <div class="grid grid-cols-2 gap-4">
        <div>
            <h2 class="mb-2 text-base-fg-muted font-medium">Agent</h2>
            <Select
                collection={() =>
                    new ListCollection({
                        items: $agents.data ?? [],
                        itemToString: (item) => item.model,
                        itemToValue: (item) => item.id,
                    })}
                value={() => ($currentAgent.data ? [$currentAgent.data.id] : undefined)}
                onSelect={async (details) => {
                    await invoke('update_current_agent', { agentId: details.value });
                    await queryClient.invalidateQueries({ queryKey: ['current-agent'] });
                }}
                bind:this={select}
            >
                {#snippet children(api)}
                    <div {...api.getRootProps()}>
                        <div {...api.getControlProps()}>
                            <button
                                {...api.getTriggerProps()}
                                class="px-2 py-1 min-w-64 w-full text-left bg-base-light dark:bg-base-dark border border-base-border
                                hover:bg-base-hover data-[state=open]:bg-base-dark
                                focus-visible:outline-none focus-visible:ring focus-visible:ring-offset-2 focus-visible:ring-offset-base focus-visible:ring-base-border"
                            >
                                <label {...api.getLabelProps()} class="block c-label">
                                    Model
                                </label>
                                <span>
                                    {api.valueAsString || 'Select agent'}
                                </span>
                            </button>
                        </div>
                        <div use:portal {...api.getPositionerProps()}>
                            <ul
                                {...api.getContentProps()}
                                class="w-(--reference-width) bg-base-light dark:bg-base-dark min-w-max border border-base-border p-1 focus:outline-none"
                            >
                                {#each $agents.data ?? [] as item (item.id)}
                                    <li
                                        {...api.getItemProps({ item })}
                                        class="px-2 py-1 data-[highlighted]:bg-base-hover"
                                    >
                                        <span {...api.getItemTextProps({ item })}>{item.model}</span
                                        >
                                        <span {...api.getItemIndicatorProps({ item })}>✓</span>
                                    </li>
                                {/each}
                            </ul>
                        </div>
                    </div>
                {/snippet}
            </Select>
            <div class="mt-4 flex flex-col gap-2">
                {#if selectApi?.hasSelectedItems}
                    {@const item = selectApi.selectedItems[0]}
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
        {#if selectApi?.hasSelectedItems}
            {@const item = selectApi.selectedItems[0]}
            {#if item}
                <div>
                    <h2 class="mb-2 text-base-fg-muted font-medium">Parameters</h2>
                    {#if item.provider === 'gemini'}
                        <div
                            class="border border-base-border bg-base-light dark:bg-base-dark px-2 py-1
                        focus-within:outline-none focus-within:ring focus-within:ring-offset-2 focus-within:ring-offset-base focus-within:ring-base-border"
                        >
                            <label for="api_key" class="block c-label">API key</label>
                            <input
                                id="api_key"
                                type="text"
                                placeholder="Enter Gemini API key"
                                class="w-full focus:outline-none placeholder:text-base-fg-muted"
                                value={$agentConfig.data?.api_key ?? undefined}
                                onblur={async (e) => {
                                    if (
                                        e.currentTarget.value === ($agentConfig.data?.api_key ?? '')
                                    ) {
                                        return;
                                    }
                                    await invoke('upsert_agent_config', {
                                        id: item.id,
                                        upsert: {
                                            api_key: e.currentTarget.value,
                                        },
                                    });
                                }}
                            />
                        </div>
                    {/if}
                </div>
            {/if}
        {/if}
    </div>
</div>
