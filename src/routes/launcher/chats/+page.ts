import type { ChatMessage } from '$lib/common/models';
import { invoke } from '@tauri-apps/api/core';
import type { PageLoad } from './$types';
import { persisted } from '../persisted.svelte';

export const load: PageLoad = async ({ parent }) => {
    const chat = persisted.chat;
    if (chat) {
        const { queryClient } = await parent();
        await queryClient.prefetchQuery({
            queryKey: ['chat-messages', { chatId: chat.id }],
            queryFn: () => {
                return invoke<ChatMessage[]>('get_chat_messages', { chatId: chat.id });
            },
        });
    }
};
