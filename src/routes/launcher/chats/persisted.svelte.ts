import type { ChatMessage } from "$lib/common/models";

let messages = $state.raw<(ChatMessage & { html: string })[]>();

export const persisted = {
    get messages() {
        return messages;
    },
    set messages(value) {
        messages = value;
    }
}
