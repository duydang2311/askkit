import type { ChatMessage } from '$lib/common/models';

let chat = $state.raw<{
    id: string;
    messages: (ChatMessage & { html: string })[];
} | null>(null);

export const persisted = {
    get chat() {
        return chat;
    },
    set chat(value) {
        chat = value;
    },
};
