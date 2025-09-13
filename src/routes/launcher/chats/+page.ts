import type { ChatMessage } from "$lib/common/models";
import { invoke } from "@tauri-apps/api/core";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ parent }) => {
    const { queryClient } = await parent();
    const chatId = localStorage.getItem('active_chat_id');
    if (chatId) {
        await queryClient.prefetchQuery({
            queryKey: ['chat-messages', { chatId }],
            queryFn: () => {
                return invoke<ChatMessage[]>('get_chat_messages', { chatId });
            }
        })
    }
};
