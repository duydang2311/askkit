import { createQuery } from '@tanstack/svelte-query';
import { invoke } from '@tauri-apps/api/core';
import { derived } from 'svelte/store';
import type { Agent } from './models';

export const useCurrentAgent = () => {
    return createQuery({
        queryKey: ['current-agent'],
        queryFn: () => invoke<Agent | null>('get_current_agent'),
    });
};

export const useAgents = () => {
    return createQuery({
        queryKey: ['agents'],
        queryFn: () => invoke<Agent[]>('get_agents'),
    });
};

export const useCurrentAgentConfig = () => {
    return createQuery(
        derived(useCurrentAgent(), ($currentAgent) => ({
            enabled: $currentAgent.data != null,
            queryKey: ['agent-config', { id: $currentAgent.data?.id }],
            queryFn: () =>
                invoke<{ agent_id: string; api_key: string } | null>('get_agent_config', {
                    id: $currentAgent.data!.id,
                }),
        }))
    );
};
