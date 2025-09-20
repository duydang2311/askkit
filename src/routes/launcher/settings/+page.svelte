<script lang="ts">
    import { AgentProvider } from '$lib/common/models';
    import { useAgents, useCurrentAgent } from '$lib/common/queries';
    import { useRuntime } from '$lib/common/runtime';
    import { button } from '$lib/common/styles';
    import { createSelect } from '$lib/components/builders.svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { ListCollection } from '@zag-js/collection';
    import { portal } from '@zag-js/svelte';
    import Google from './Google.svelte';
    import Groq from './Groq.svelte';

    const { queryClient } = useRuntime();

    const agents = useAgents();
    const currentAgent = useCurrentAgent();

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
</script>

<div class="px-6 py-4">
    <div class="grid grid-cols-2 gap-4">
        <div>
            <h2 class="text-base-fg-muted mb-2 font-medium">Agent</h2>
            <div {...select.getRootProps()}>
                <div {...select.getControlProps()}>
                    <button
                        {...select.getTriggerProps()}
                        class="{button({ filled: true, border: true })} w-full text-left"
                    >
                        <label {...select.getLabelProps()} class="c-label block">Model</label>
                        <span>
                            {select.valueAsString || 'Select agent'}
                        </span>
                    </button>
                </div>
                <div use:portal {...select.getPositionerProps()}>
                    <ul
                        {...select.getContentProps()}
                        class="bg-base-light dark:bg-base-dark border-base-border w-(--reference-width) min-w-max border p-1 focus:outline-none"
                    >
                        {#each $agents.data ?? [] as item (item.id)}
                            <li
                                {...select.getItemProps({ item })}
                                class="data-[highlighted]:bg-base-hover px-2 py-1"
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
                    <h2 class="text-base-fg-muted mb-2 font-medium">Parameters</h2>
                    {#if item.provider === AgentProvider.Google}
                        <Google agent={item} bind:showApiKey />
                    {:else if item.provider === AgentProvider.Groq}
                        <Groq agent={item} bind:showApiKey />
                    {/if}
                </div>
            {/if}
        {/if}
    </div>
</div>
