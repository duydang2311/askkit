import { createQuery } from '@tanstack/svelte-query';
import { invoke } from '@tauri-apps/api/core';
import type { Agent, AgentProvider } from './models';

export const useCurrentAgent = () => {
    return createQuery({
        queryKey: ['current-agent'],
        queryFn: () => invoke<Agent | null>('get_current_agent'),
    });
};

export const useAgents = () => {
    return createQuery({
        queryKey: ['agents'],
        queryFn: () =>
            invoke<
                {
                    id: string;
                    provider: AgentProvider;
                    model: string;
                }[]
            >('get_agents'),
    });
};
