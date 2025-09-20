<script lang="ts">
    import type { Agent } from '$lib/common/models';
    import { useCurrentAgentConfig } from '$lib/common/queries';
    import { useRuntime } from '$lib/common/runtime';
    import { createPasswordInput } from '$lib/components/builders.svelte';
    import { Eye, EyeSlash } from '$lib/components/icons';
    import { attempt } from '@duydang2311/attempt';
    import { invoke } from '@tauri-apps/api/core';

    let { agent, showApiKey = $bindable() }: { agent: Agent; showApiKey: boolean } = $props();

    const { queryClient } = useRuntime();
    const agentConfig = useCurrentAgentConfig();
    const id = $props.id();

    let decryptedApiKey = $state.raw<string | null>(null);
    const passwordInput = createPasswordInput({
        id,
        get visible() {
            return showApiKey;
        },
        onVisibilityChange: async (details) => {
            let decryptedStr: string | null = null;
            if (details.visible) {
                const ciphertext = $agentConfig.data?.api_key;
                if (ciphertext) {
                    const decrypted = await attempt.async(() =>
                        invoke<string>('decrypt_agent_ciphertext', {
                            ciphertext,
                        })
                    )(console.error);
                    if (decrypted.ok) {
                        decryptedStr = decrypted.data;
                    }
                }
            }
            decryptedApiKey = decryptedStr;
            showApiKey = details.visible;
        },
    });
</script>

<div
    {...passwordInput.getRootProps()}
    class={[
        'border-base-border bg-base-light dark:bg-base-dark border px-2 py-1',
        'focus-within:ring-offset-base focus-within:ring-base-border focus-within:ring focus-within:ring-offset-2 focus-within:outline-none',
    ]}
>
    <label {...passwordInput.getLabelProps()} class="c-label block"> API key </label>
    <div {...passwordInput.getControlProps()} class="flex gap-2">
        <input
            {...passwordInput.getInputProps()}
            placeholder="Enter Groq API key"
            value={showApiKey ? decryptedApiKey : ($agentConfig.data?.api_key ?? undefined)}
            class="placeholder:text-base-fg-muted w-full focus:outline-none"
            onblur={async (e) => {
                if (e.currentTarget.value === ($agentConfig.data?.api_key ?? '')) {
                    return;
                }
                await invoke('upsert_agent_config', {
                    id: agent.id,
                    upsert: {
                        api_key: e.currentTarget.value,
                    },
                });
                await queryClient.invalidateQueries({
                    queryKey: ['agent-config', { id: agent.id }],
                });
            }}
        />
        <button {...passwordInput.getVisibilityTriggerProps()}>
            <span {...passwordInput.getIndicatorProps()}>
                {#if passwordInput.visible}
                    <Eye />
                {:else}
                    <EyeSlash />
                {/if}
            </span>
        </button>
    </div>
</div>
