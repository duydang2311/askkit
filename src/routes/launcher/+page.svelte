<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { LogicalSize } from '@tauri-apps/api/dpi';
    import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
    import { Editor, Extension } from '@tiptap/core';
    import CodeBlock from '@tiptap/extension-code-block';
    import Document from '@tiptap/extension-document';
    import HardBreak from '@tiptap/extension-hard-break';
    import Paragraph from '@tiptap/extension-paragraph';
    import Text from '@tiptap/extension-text';
    import { Placeholder } from '@tiptap/extensions';
    import DOMPurify from 'dompurify';
    import { marked } from 'marked';
    import { onMount, tick } from 'svelte';
    import { on } from 'svelte/events';
    import { Markdown } from 'tiptap-markdown';
    import PaperAirplane from '~icons/heroicons/paper-airplane-16-solid';
    import BotOff from '~icons/lucide/bot-off';
    import Bot from '~icons/lucide/bot';
    import { onEvent } from '../../lib/common/tauri';
    import Greetings from './Greetings.svelte';
    import { isAppError } from '../../lib/common/error';
    import { SvelteSet } from 'svelte/reactivity';
    import AgentScreen from './AgentScreen.svelte';
    import { useCurrentAgent } from '$lib/common/queries';

    const editorBaseClass = 'w-screen max-h-64 overflow-auto pl-6 pr-28 py-4 focus:outline-none';
    const currentAgent = useCurrentAgent();
    let editor = $state.raw<Editor>();
    let containerEl = $state.raw<HTMLElement>();
    let chatEl = $state.raw<HTMLElement>();
    let chat = $state.raw<{
        id: string;
        messages: { id: string; role: string; content: string }[];
    } | null>(null);
    let errors = new SvelteSet<string>();
    let showAgentScreen = $state.raw(false);

    const hasMultilines = (editor: Editor) => {
        if (editor.state.doc.content.childCount > 1) {
            return true;
        }
        let hasMultilines = false;
        editor.state.doc.descendants((node) => {
            if (node.type.name === 'hardBreak') {
                hasMultilines = true;
                return false;
            }
        });
        return hasMultilines;
    };

    const submit = async () => {
        errors.clear();
        if (!editor) {
            return;
        }
        const content = editor.getText() ?? '';
        editor.commands.clearContent();
        if (!chat) {
            chat = {
                id: await invoke<string>('create_chat', {
                    content,
                }),
                messages: [],
            };
        }
        try {
            await invoke<string>('send_chat_message', {
                content,
                chatId: chat.id,
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
        if (chat?.id !== data.chatId) {
            return;
        }

        chat = {
            ...chat,
            messages: chat.messages.map((a) =>
                a.id === data.id
                    ? {
                          ...a,
                          content: a.content + data.text,
                      }
                    : a
            ),
        };
    });

    onEvent('chat_message_created', (e) => {
        // TODO: validate
        const data = e.payload as {
            chatId: string;
            id: string;
            role: string;
            content: string;
        };
        if (chat?.id !== data.chatId) {
            return;
        }

        chat = {
            ...chat,
            messages: [
                ...chat.messages,
                {
                    ...data,
                    content: data.content,
                },
            ],
        };

        if (chatEl) {
            const el = chatEl;
            const gapPx =
                (
                    document.documentElement.computedStyleMap().get('font-size') as
                        | CSSUnitValue
                        | undefined
                )?.value ?? 16;
            const secondLastEl = el.lastElementChild as HTMLElement | undefined;
            tick().finally(() => {
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
                        if (data.role === 'user') {
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
                                top: (lastUserRoleEl ?? lastEl).offsetTop - el.offsetTop - gapPx,
                                behavior: 'smooth',
                            });
                        }
                    });
                }
            });
        }
    });

    onEvent<string>('chat_message_rollback', (e) => {
        const id = e.payload;
        if (chat) {
            chat = {
                ...chat,
                messages: chat.messages.filter((m) => m.id !== id),
            };
        }
    });

    const updateWindowSize = (width: number, height: number) => {
        return getCurrentWebviewWindow().setSize(new LogicalSize(width, height));
    };
    let lastHeight = 0;

    onMount(() => {
        return on(window, 'keyup', async (e) => {
            if (e.key === 'Escape') {
                await invoke('destroy_launcher_window');
            }
        });
    });
</script>

<main
    bind:this={containerEl}
    {@attach (node) => {
        const observer = new ResizeObserver(() => {
            // avoid flickering due to sub-pixel rendering differences, especially when typing
            if (Math.abs(node.scrollHeight - lastHeight) <= 1) {
                return;
            }
            lastHeight = node.scrollHeight;
            updateWindowSize(node.scrollWidth, node.scrollHeight);
        });
        observer.observe(node);
        return () => {
            observer.disconnect();
        };
    }}
>
    <div class="flex justify-between gap-4 px-6 py-2 border-b border-b-base-border">
        <Greetings />
        <p class="text-primary font-bold tracking-tight">askkit</p>
    </div>
    {#if showAgentScreen}
        <AgentScreen />
    {:else if chat}
        <ol
            bind:this={chatEl}
            class="relative h-128 border-b border-b-base-border px-6 py-4 overflow-auto space-y-4 custom-scrollbar"
            {@attach () => {
                if (!containerEl) {
                    return;
                }
                getCurrentWebviewWindow().setSize(
                    new LogicalSize(containerEl.scrollWidth, containerEl.scrollHeight)
                );
            }}
        >
            {#each chat.messages as msg (msg.id)}
                {#await marked(msg.content) then html}
                    <li data-role={msg.role}>
                        <div
                            data-role={msg.role}
                            class="data-[role=user]:bg-base-light p-2 rounded-md w-fit max-w-[80ch] data-[role=user]:ml-auto wrap-anywhere prose"
                        >
                            {@html DOMPurify.sanitize(html)}
                        </div>
                    </li>
                {/await}
            {/each}
            {#if errors.size > 0}
                <div class="absolute inset-0 flex flex-col justify-center items-center p-20">
                    {#if errors.has('AgentRequiredError')}
                        {@render agentRequiredError()}
                    {:else if errors.has('AgentTextGenParamsRequiredError')}
                        {@render agentTextGenParamsRequiredError()}
                    {/if}
                </div>
            {/if}
        </ol>
    {/if}
    <div
        class="relative"
        {@attach (node) => {
            if (!containerEl) {
                return;
            }

            const container = containerEl;
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
                            await updateWindowSize(container.scrollWidth, container.scrollHeight);
                            props.editor.view.setProps({
                                attributes: {
                                    class: editorBaseClass,
                                    style: `height: ${node.scrollHeight}px`,
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
            class="flex items-center gap-2 absolute right-6 bottom-4 not-[[data-multiline]]:bottom-1/2 not-[[data-multiline]]:translate-y-1/2 z-10"
        >
            <button
                type="button"
                disabled={editor?.isEmpty ?? true}
                class="p-1 size-8 text-primary disabled:text-base-fg-muted"
                onclick={submit}
            >
                <PaperAirplane class="size-full" />
            </button>
        </div>
    </div>
    <div class="px-6 py-2 flex items-center gap-2 border-t border-t-base-border">
        <button
            id="agent"
            type="button"
            class="p-1 size-8 disabled:text-base-fg-muted"
            onclick={() => {
                showAgentScreen = !showAgentScreen;
            }}
        >
            <Bot class="size-full" />
        </button>
        {#if $currentAgent.data}
            <label
                for="agent"
                class="bg-base-dark dark:bg-base-light text-base-fg-muted text-xs p-1 leading-none content-center"
            >
                {$currentAgent.data.model}
            </label>
        {/if}
    </div>
</main>

{#snippet agentRequiredError()}
    <BotOff class="block size-16 text-base-fg-muted" />
    <p class="text-base-fg-muted text-xl">No agent selected</p>
    <p class="mt-4">Please select an AI agent to start chatting</p>
{/snippet}

{#snippet agentTextGenParamsRequiredError()}
    <BotOff class="block size-16 text-base-fg-muted" />
    <p class="text-base-fg-muted text-xl">Agent not configured</p>
    <p class="mt-4">Configure the required parameters for the selected agent to start chatting</p>
{/snippet}
