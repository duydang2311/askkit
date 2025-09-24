<!--  --><script lang="ts">
    import { ChatMessageStatus, type ChatMessage } from '$lib/common/models';
    import { useCurrentAgent } from '$lib/common/queries';
    import { useRuntime } from '$lib/common/runtime';
    import { button } from '$lib/common/styles';
    import { attempt } from '@duydang2311/attempt';
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
    import { onMount, untrack } from 'svelte';
    import { on } from 'svelte/events';
    import { Spring } from 'svelte/motion';
    import { SvelteSet } from 'svelte/reactivity';
    import { toStore } from 'svelte/store';
    import { Markdown } from 'tiptap-markdown';
    import PaperAirplane from '~icons/heroicons/paper-airplane-16-solid';
    import BotOff from '~icons/lucide/bot-off';
    import { isAppError } from '../../../lib/common/error';
    import { onEvent } from '../../../lib/common/tauri';
    import { persisted } from '../persisted.svelte';

    const editorBaseClass =
        'w-screen max-h-64 overflow-auto pl-6 pr-28 py-4 focus:outline-none caret-transparent';
    const { queryClient } = useRuntime();
    let editor = $state.raw<Editor>();
    let messagesContainerEl = $state.raw<HTMLElement>();
    let errors = new SvelteSet<string>();

    const chatMessagesQueryKey = $derived(['chat-messages', { chatId: persisted.chat?.id }]);
    const chatMessages = createQuery(
        toStore(() => ({
            enabled: persisted.chat != null,
            queryKey: chatMessagesQueryKey,
            queryFn: () => invoke<ChatMessage[]>('get_chat_messages', { id: persisted.chat!.id }),
        }))
    );
    const currentAgent = useCurrentAgent();
    const latestMessage = $derived(persisted.chat?.messages.at(-1));

    watch(() => $chatMessages.data)(async () => {
        if (persisted.chat == null) {
            return;
        }

        marked.use({
            renderer: {
                link(link) {
                    const token = this.parser.parseInline(link.tokens);
                    return `<a href="${link.href}" target="_blank" rel="noopener noreferrer">${token}</a>`;
                },
            },
        });
        persisted.chat.messages =
            $chatMessages.data == null
                ? []
                : await Promise.all(
                      $chatMessages.data.map(async (a) => {
                          const html = marked(a.content);
                          return {
                              ...a,
                              html:
                                  html instanceof Promise
                                      ? await html.then((a) =>
                                            DOMPurify.sanitize(a, { ADD_ATTR: ['target'] })
                                        )
                                      : DOMPurify.sanitize(html, { ADD_ATTR: ['target'] }),
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

        persisted.chat ??= {
            id: await invoke<string>('create_chat', {
                content,
            }),
            messages: [],
        };

        const chat = persisted.chat;
        const sent = await attempt.async(() =>
            invoke<string>('send_chat_message', {
                content,
                chatId: chat.id,
            })
        )();

        if (sent.failed) {
            if (isAppError(sent.error)) {
                if (
                    sent.error.kind === 'AgentRequiredError' ||
                    sent.error.kind === 'AgentTextGenParamsRequiredError'
                ) {
                    errors.add(sent.error.kind);
                }
            } else {
                throw sent.error;
            }
        }
    };

    let caretEl = $state.raw<HTMLDivElement>();
    let inputContainerEl = $state.raw<HTMLElement>();
    let caretLh = 0.85;
    let caretHeight: number | null = null;
    let springCaretWidth = new Spring(12, { damping: 0.6, stiffness: 0.3 });
    let blinkTimeout = 0;
    const springTop = new Spring(0, { damping: 0.6, stiffness: 0.25 });
    const springLeft = new Spring(0, { damping: 0.6, stiffness: 0.25 });
    const updateCaret = (editor: Editor, instant?: boolean) => {
        if (!caretEl) {
            return;
        }

        caretHeight ??= parseFloat(getComputedStyle(caretEl).lineHeight) * caretLh;

        const from = editor.state.selection.from;
        const fromCoords = editor.view.coordsAtPos(
            Math.min(from, editor.state.doc.content.size - 1)
        );
        console.log('fromCoords', fromCoords);
        let top =
            fromCoords.top +
            (fromCoords.bottom - fromCoords.top - caretHeight) / 2 -
            (inputContainerEl?.offsetTop ?? 0);
        let left = fromCoords.left;

        top += window.visualViewport?.offsetTop ?? 0;
        left += window.visualViewport?.offsetLeft ?? 0;

        springTop.set(top, {
            instant,
        });
        springLeft.set(left, { instant });
        if (!editor.state.selection.empty) {
            springCaretWidth.set(0, { instant: true });
        } else {
            springCaretWidth.set(from >= editor.state.doc.content.size - 1 ? 12 : 2);
        }

        caretEl?.classList.remove('animate-caret-blink', 'animate-caret-pop');
        requestAnimationFrame(() => {
            caretEl?.classList.add('animate-caret-pop');
        });
        if (blinkTimeout) {
            clearTimeout(blinkTimeout);
        }
        blinkTimeout = setTimeout(() => {
            caretEl?.classList.remove('animate-caret-pop');
            if (editor.isFocused) {
                caretEl?.classList.add('animate-caret-blink');
            }
        }, 400);
    };

    onEvent('chat_message_response_chunk', (e) => {
        // TODO: validate
        const data = e.payload as { chatId: string; id: string; text: string };
        if (persisted.chat?.id !== data.chatId) {
            return;
        }

        queryClient.setQueryData<ChatMessage[]>(
            ['chat-messages', { chatId: persisted.chat.id }],
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
        if (persisted.chat?.id !== msg.chatId) {
            return;
        }

        if (queryClient.isFetching({ queryKey: chatMessagesQueryKey })) {
            await queryClient.cancelQueries({ queryKey: chatMessagesQueryKey });
        }
        queryClient.setQueryData<ChatMessage[]>(chatMessagesQueryKey, (messages) => {
            return [...(messages ?? []), msg];
        });

        if (messagesContainerEl) {
            const el = messagesContainerEl;
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
        if (persisted.chat?.id !== e.payload.chatId) {
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
            if (persisted.chat?.id !== e.payload.chatId) {
                return;
            }

            if (queryClient.isFetching({ queryKey: chatMessagesQueryKey })) {
                await queryClient.cancelQueries({ queryKey: chatMessagesQueryKey });
            }
            queryClient.setQueryData<ChatMessage[]>(chatMessagesQueryKey, (messages) => {
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
    {#if persisted.chat && persisted.chat.messages.length > 0}
        <ol
            bind:this={messagesContainerEl}
            class="custom-scrollbar h-128 space-y-4 overflow-auto px-6 py-4"
        >
            {#each persisted.chat.messages as msg (msg.id)}
                <li data-role={msg.role}>
                    <div
                        data-role={msg.role}
                        class="data-[role=user]:bg-base-light prose w-fit max-w-[80ch] rounded p-2 wrap-anywhere data-[role=user]:ml-auto"
                    >
                        {@html msg.html}
                    </div>
                </li>
            {/each}
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
    {/if}
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
    bind:this={inputContainerEl}
    class="border-base-border relative overflow-hidden border-t"
    {@attach (node) => {
        const currentEditor = new Editor({
            element: node,
            editorProps: {
                attributes: { spellcheck: 'false', class: editorBaseClass },
            },
            extensions: [
                Document,
                Text,
                Paragraph,
                HardBreak,
                Placeholder.configure({
                    placeholder: 'Start typing...',
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
                }),
            ],
            onTransaction: (props) => {
                untrack(() => {
                    editor = undefined;
                    editor = props.editor;
                    updateCaret(editor);
                });
            },
        });
        currentEditor.commands.focus();
        const off = on(currentEditor.view.dom, 'scroll', () => {
            updateCaret(currentEditor, true);
        });
        untrack(() => {
            editor = currentEditor;
        });
        return () => {
            off();
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
            class="{button({ variant: 'primary' })} size-8 p-1"
            onclick={submit}
        >
            <PaperAirplane class="size-full" />
        </button>
    </div>
    <div
        bind:this={caretEl}
        class={[
            'absolute transition-colors',
            editor?.isFocused ? 'bg-primary animate-caret-blink' : 'bg-transparent',
        ]}
        style="width: {springCaretWidth.current}px; height: {caretLh}lh; top: {springTop.current}px; left: {springLeft.current}px;"
    ></div>
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

<style>
    @keyframes caret-pop {
        0% {
            transform: scale(1);
            opacity: 1;
        }
        50% {
            transform: scale(0.9);
            opacity: 0.8;
        }
        100% {
            transform: scale(1);
            opacity: 1;
        }
    }

    @keyframes caret-blink {
        0% {
            opacity: 1;
        }
        20% {
            opacity: 0.4;
        }
        40% {
            opacity: 1;
        }
        60% {
            opacity: 0.4;
        }
        80% {
            opacity: 1;
        }
    }

    .animate-caret-blink {
        animation: caret-blink 3s ease infinite;
    }

    :global(.animate-caret-pop) {
        animation: caret-pop 100ms ease;
    }

    :global(.animate-caret-blink.animate-caret-pop) {
        animation:
            caret-pop 100ms ease,
            caret-blink 3s ease infinite alternate;
    }

    :global(.tiptap::selection) {
        background-color: var(--color-primary);
        color: var(--color-primary-fg);
    }

    @layer base {
        :global(p.is-editor-empty:first-child::before) {
            color: color-mix(in oklch, var(--color-base-fg) 40%, transparent);
            content: attr(data-placeholder);
            float: left;
            height: 0;
            pointer-events: none;
        }
    }
</style>
