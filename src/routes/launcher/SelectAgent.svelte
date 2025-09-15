<script lang="ts">
    import { useAgents, useCurrentAgent } from '$lib/common/queries';
    import { useRuntime } from '$lib/common/runtime';
    import { createSelect } from '$lib/components/builders.svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { ListCollection } from '@zag-js/collection';
    import { portal } from '@zag-js/svelte';

    const { queryClient } = useRuntime();
    const id = $props.id();
    const agents = useAgents();
    const currentAgent = useCurrentAgent();
    const select = createSelect({
        id,
        get collection() {
            return new ListCollection({
                items: $agents.data ?? [],
                itemToString: (a) => a.model,
                itemToValue: (a) => a.id,
            });
        },
        get value() {
            return $currentAgent.data ? [$currentAgent.data.id] : undefined;
        },
        onSelect: async (details) => {
            await invoke('update_current_agent', { agentId: details.value });
            await queryClient.invalidateQueries({ queryKey: ['current-agent'] });
        },
    });
</script>

<div {...select.getRootProps()}>
    <div {...select.getControlProps()} class="h-full">
        <button
            {...select.getTriggerProps()}
            class="text-base-fg-muted bg-base-dark hover:bg-base-hover data-[state=open]:bg-base-dark focus-visible:ring-offset-base focus-visible:ring-base-border h-full px-2 py-1 text-left text-xs focus-visible:ring focus-visible:ring-offset-2 focus-visible:outline-none"
        >
            <span>
                {$currentAgent.data?.model ?? 'Select agent'}
            </span>
        </button>
    </div>
    <div use:portal {...select.getPositionerProps()}>
        <ul
            {...select.getContentProps()}
            class="bg-base-light dark:bg-base-dark border-base-border w-(--reference-width) min-w-max border p-1 text-xs focus:outline-none"
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
