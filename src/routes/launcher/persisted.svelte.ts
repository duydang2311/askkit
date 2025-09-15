import type { ChatMessage } from '$lib/common/models';

let chatId = $state.raw<string | null>(null);
let messages = $state.raw<(ChatMessage & { html: string })[] | null>(null);

export const persisted = {
    get chatId() {
        return chatId;
    },
    set chatId(value) {
        chatId = value;
    },
    get messages() {
        return messages;
    },
    set messages(value) {
        messages = value;
    },
};
