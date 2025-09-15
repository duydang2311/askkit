<script lang="ts">
    import { ChatMessageStatus, type ChatMessage } from '$lib/common/models';
    import { useCurrentAgent } from '$lib/common/queries';
    import { useRuntime } from '$lib/common/runtime';
    import { watch } from '@duydang2311/svutils';
    import { createQuery } from '@tanstack/svelte-query';
    import { invoke } from '@tauri-apps/api/core';
    import { Editor, Extension } from '@tiptap/core';
    import CodeBlock from '@tiptap/extension-code-block';
    import Document from '@tiptap/extension-document';
    import HardBreak from '@tiptap/extension-hard-break';
    import Paragraph from '@tiptap/extension-paragraph';
    import Text from '@tiptap/extension-text';
    import { Placeholder } from '@tiptap/extensions';
    import DOMPurify from 'dompurify';
    import { marked } from 'marked';
    import { onMount } from 'svelte';
    import { on } from 'svelte/events';
    import { SvelteSet } from 'svelte/reactivity';
    import { toStore } from 'svelte/store';
    import { Markdown } from 'tiptap-markdown';
    import PaperAirplane from '~icons/heroicons/paper-airplane-16-solid';
    import BotOff from '~icons/lucide/bot-off';
    import { isAppError } from '../../../lib/common/error';
    import { onEvent } from '../../../lib/common/tauri';
    import { persisted } from './persisted.svelte';

    const editorBaseClass = 'w-screen max-h-64 overflow-auto pl-6 pr-28 py-4 focus:outline-none';
    const { queryClient } = useRuntime();
    let editor = $state.raw<Editor>();
    let chatEl = $state.raw<HTMLElement>();
    let errors = new SvelteSet<string>();

    const chatMessagesQueryKey = $derived(['chat-messages', { chatId: persisted.chatId }]);
    const chatMessages = createQuery(
        toStore(() => ({
            enabled: persisted.chatId != null,
            queryKey: chatMessagesQueryKey,
            queryFn: () => invoke<ChatMessage[]>('get_chat_messages', { id: persisted.chatId! }),
        }))
    );
    const currentAgent = useCurrentAgent();
    const latestMessage = $derived(persisted.messages?.at(-1));

    watch(() => $chatMessages.data)(async () => {
        persisted.messages =
            $chatMessages.data == null
                ? undefined
                : await Promise.all(
                      $chatMessages.data.map(async (a) => {
                          const html = marked(a.content);
                          return {
                              ...a,
                              html:
                                  html instanceof Promise
                                      ? await html.then(DOMPurify.sanitize)
                                      : DOMPurify.sanitize(html),
                          };
                      })
                  );
    });

    const hasMultilines = (editor: Editor) => {
        if (editor.state.doc.content.childCount > 1) {
            return true;
        }
        let has = false;
        editor.state.doc.descendants((node) => {
            if (node.type.name === 'hardBreak') {
                has = true;
                return false;
            }
        });
        return has;
    };

    const submit = async () => {
        errors.clear();
        if (!editor) {
            return;
        }
        const content = editor.getText() ?? '';
        editor.commands.clearContent();
        if (!persisted.chatId) {
            persisted.chatId = await invoke<string>('create_chat', {
                content,
            });
        }
        try {
            await invoke<string>('send_chat_message', {
                content,
                chatId: persisted.chatId,
            });
        } catch (e) {
            console.log(e);
            if (isAppError(e)) {
                if (e.kind === 'AgentRequiredError' || e.kind === 'AgentTextGenParamsRequiredError')
                    errors.add(e.kind);
            } else {
                throw e;
            }
        }
    };

    onEvent('chat_message_response_chunk', (e) => {
        // TODO: validate
        const data = e.payload as { chatId: string; id: string; text: string };
        if (persisted.chatId !== data.chatId) {
            return;
        }

        queryClient.setQueryData<ChatMessage[]>(
            ['chat-messages', { chatId: persisted.chatId }],
            (messages) => {
                return messages?.map((a) =>
                    a.id === data.id
                        ? {
                              ...a,
                              content: a.content + data.text,
                          }
                        : a
                );
            }
        );
    });

    onEvent('chat_message_created', async (e) => {
        // TODO: validate
        const msg = e.payload as ChatMessage;
        if (persisted.chatId !== msg.chatId) {
            return;
        }

        if (queryClient.isFetching({ queryKey: chatMessagesQueryKey })) {
            await queryClient.cancelQueries({ queryKey: chatMessagesQueryKey });
        }
        queryClient.setQueryData<ChatMessage[]>(chatMessagesQueryKey, (messages) => {
            return [...(messages ?? []), msg];
        });

        if (chatEl) {
            const el = chatEl;
            const gapPx =
                (
                    document.documentElement.computedStyleMap().get('font-size') as
                        | CSSUnitValue
                        | undefined
                )?.value ?? 16;
            const secondLastEl = el.lastElementChild as HTMLElement | undefined;
            setTimeout(() => {
                if (el.scrollHeight <= el.clientHeight) {
                    return;
                }
                const lastEl = el.lastElementChild as HTMLElement;
                if (lastEl) {
                    lastEl.style.minHeight = el.clientHeight - 2 * gapPx + 'px';
                    if (secondLastEl) {
                        secondLastEl.style.minHeight = '';
                    }
                    requestAnimationFrame(() => {
                        if (msg.role === 'user') {
                            el.scrollTo({
                                top: el.scrollHeight,
                                behavior: 'smooth',
                            });
                        } else {
                            let lastUserRoleEl: HTMLElement | null = lastEl;
                            while (
                                lastUserRoleEl != null &&
                                lastUserRoleEl.getAttribute('data-role') !== 'user'
                            ) {
                                lastUserRoleEl =
                                    lastUserRoleEl.previousElementSibling as HTMLElement | null;
                            }
                            el.scrollTo({
                                top: (lastUserRoleEl ?? lastEl).offsetTop - gapPx,
                                behavior: 'smooth',
                            });
                        }
                    });
                }
            }, 100);
        }
    });

    onEvent<{ chatId: string; messageId: string }>('chat_message_rollback', async (e) => {
        if (persisted.chatId !== e.payload.chatId) {
            return;
        }

        if (queryClient.isFetching({ queryKey: chatMessagesQueryKey })) {
            await queryClient.cancelQueries({ queryKey: chatMessagesQueryKey });
        }
        queryClient.setQueryData<ChatMessage[]>(chatMessagesQueryKey, (messages) => {
            return messages?.filter((m) => m.id !== e.payload.messageId);
        });
    });

    onEvent<{ chatId: string; messageId: string; status: ChatMessageStatus }>(
        'chat_message_status_changed',
        async (e) => {
            if (persisted.chatId !== e.payload.chatId) {
                return;
            }

            if (queryClient.isFetching({ queryKey: chatMessagesQueryKey })) {
                await queryClient.cancelQueries({ queryKey: chatMessagesQueryKey });
            }
            queryClient.setQueryData<ChatMessage[]>(chatMessagesQueryKey, (messages) => {
                console.log(messages);
                return messages?.map((m) =>
                    m.id === e.payload.messageId ? { ...m, status: e.payload.status } : m
                );
            });
        }
    );

    onMount(() => {
        return on(window, 'keyup', async (e) => {
            if (e.key === 'Escape') {
                await invoke('destroy_launcher_window');
            }
        });
    });
</script>

<div class="relative">
    <ol bind:this={chatEl} class="custom-scrollbar h-128 space-y-4 overflow-auto px-6 py-4">
        {#if persisted.messages}
            {#each persisted.messages as msg (msg.id)}
                <li data-role={msg.role}>
                    <div
                        data-role={msg.role}
                        class="data-[role=user]:bg-base-light prose w-fit max-w-[80ch] rounded p-2 wrap-anywhere data-[role=user]:ml-auto"
                    >
                        {@html msg.html}
                    </div>
                </li>
            {/each}
        {/if}
        {#if errors.size > 0}
            <div class="absolute inset-0 flex flex-col items-center justify-center p-20">
                {#if errors.has('AgentRequiredError')}
                    {@render agentRequiredError()}
                {:else if errors.has('AgentTextGenParamsRequiredError')}
                    {@render agentTextGenParamsRequiredError()}
                {/if}
            </div>
        {/if}
    </ol>
    {#if latestMessage?.status === ChatMessageStatus.Pending}
        <div
            class="from-base absolute inset-x-0 bottom-0 bg-gradient-to-t from-80% to-transparent px-6 pt-6 pb-2"
        >
            <span class="c-label animate-pulse">
                {$currentAgent.data?.model ?? 'Agent'} is typing...
            </span>
        </div>
    {/if}
</div>
<div
    class="border-base-border relative border-t"
    {@attach (node) => {
        const currentEditor = new Editor({
            element: node,
            extensions: [
                Document,
                Text,
                Paragraph,
                HardBreak,
                Placeholder.configure({
                    placeholder: 'Enter your questions...',
                }),
                CodeBlock,
                Markdown.configure({
                    breaks: true,
                    transformPastedText: true,
                }),
                Extension.create({
                    addKeyboardShortcuts() {
                        return {
                            'Ctrl-Enter': () => {
                                submit();
                                return true;
                            },
                        };
                    },
                    onTransaction: async (props) => {
                        props.editor.view.setProps({
                            attributes: {
                                class: editorBaseClass,
                            },
                        });
                    },
                }),
            ],
            onTransaction: (props) => {
                editor = undefined;
                editor = props.editor;
            },
        });
        currentEditor.commands.focus();
        editor = currentEditor;
        return () => {
            currentEditor.destroy();
        };
    }}
>
    <div
        data-multiline={(editor ? hasMultilines(editor) : false) ? '' : undefined}
        class="absolute right-6 bottom-4 z-10 flex items-center gap-2 not-[[data-multiline]]:bottom-1/2 not-[[data-multiline]]:translate-y-1/2"
    >
        <button
            type="button"
            disabled={editor?.isEmpty ?? true}
            class="text-primary disabled:text-base-fg-muted size-8 p-1"
            onclick={submit}
        >
            <PaperAirplane class="size-full" />
        </button>
    </div>
</div>

{#snippet agentRequiredError()}
    <BotOff class="text-base-fg-muted block size-16" />
    <p class="text-base-fg-muted text-xl">No agent selected</p>
    <p class="mt-4">Please select an AI agent to start chatting</p>
{/snippet}

{#snippet agentTextGenParamsRequiredError()}
    <BotOff class="text-base-fg-muted block size-16" />
    <p class="text-base-fg-muted text-xl">Agent not configured</p>
    <p class="mt-4">Configure the required parameters for the selected agent to start chatting</p>
{/snippet}
