import { getContext, setContext } from "svelte";

export interface LauncherContext {
    chatId?: string;
}

const symbol = Symbol();

export const setLauncherContext = (context: LauncherContext) => {
    let chatId = $state.raw(context.chatId);

    return setContext(symbol, {
        get chatId() {
            return chatId;
        },
        set chatId(value) {
            chatId = value;
        }
    });
}

export const useLauncherContext = () => {
    return getContext<LauncherContext>(symbol);
}