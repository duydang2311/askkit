import { createQuery } from '@tanstack/svelte-query';
import { invoke } from '@tauri-apps/api/core';
import type { Agent } from './models';

export const useCurrentAgent = () => {
    return createQuery({
        queryKey: ['current-agent'],
        queryFn: () => invoke<Agent | null>('get_current_agent'),
    });
}