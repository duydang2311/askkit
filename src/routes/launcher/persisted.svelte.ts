import type { ChatMessage } from '$lib/common/models';

let chat = $state<{
    id: string;
    messages: (ChatMessage & { html: string })[];
} | null>(null);
let scrollEl = $state.raw<HTMLElement | null>(null);

export const persisted = {
    get chat() {
        return chat;
    },
    set chat(value) {
        chat = value;
    },
    get scrollEl() {
        return scrollEl;
    },
    set scrollEl(value) {
        scrollEl = value;
    },
};
