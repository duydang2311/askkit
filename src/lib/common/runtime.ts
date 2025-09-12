import type { QueryClient } from "@tanstack/svelte-query";
import { getContext, setContext } from "svelte";

interface Runtime {
    queryClient: QueryClient;
}

const symbol = Symbol();

export const setRuntime = (runtime: Runtime) => {
    return setContext(symbol, runtime);
}

export const useRuntime = () => {
    return getContext<Runtime>(symbol);
}