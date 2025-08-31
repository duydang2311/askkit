<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { LogicalSize } from '@tauri-apps/api/dpi';
    import { listen } from '@tauri-apps/api/event';
    import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
    import { Editor, Extension } from '@tiptap/core';
    import CodeBlock from '@tiptap/extension-code-block';
    import Document from '@tiptap/extension-document';
    import HardBreak from '@tiptap/extension-hard-break';
    import Paragraph from '@tiptap/extension-paragraph';
    import Text from '@tiptap/extension-text';
    import { Placeholder } from '@tiptap/extensions';
    import { onMount } from 'svelte';
    import { Markdown } from 'tiptap-markdown';
    import PaperAirplane from '~icons/heroicons/paper-airplane-16-solid';
    import Greetings from './Greetings.svelte';

    const editorBaseClass =
        'w-screen max-h-64 overflow-auto pl-6 pr-18 py-4 focus:outline-none';
    let editor = $state.raw<Editor>();
    let containerEl = $state.raw<HTMLElement>();

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
        const chatId = await invoke<string>('submit_agent_prompt', {
            content: editor?.getText(),
        });
      console.log('chatId', chatId);
    };

    let answer = '';
    onMount(() => {
        const unlistenFn = listen('agent-response-chunk', (e) => {
            console.log('response-chunk', e);
            answer =
                answer +
                JSON.parse(e.payload).candidates[0].content.parts[0].text.replace(/\\\write\n/g, '\n');
            editor?.commands.setContent(answer);
        });
        return () => {
            unlistenFn.then((unlisten) => unlisten());
        };
    });
</script>

<main bind:this={containerEl}>
    <div
        class="flex justify-between gap-4 px-6 py-2 border-b border-b-base-border"
    >
        <Greetings />
        <p class="text-primary font-bold tracking-tight">askkit</p>
    </div>
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
                            await getCurrentWebviewWindow().setSize(
                                new LogicalSize(
                                    container.scrollWidth,
                                    container.scrollHeight
                                )
                            );
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
        <button
            type="button"
            disabled={editor?.isEmpty ?? true}
            data-multiline={(editor ? hasMultilines(editor) : false)
                ? ''
                : undefined}
            class="p-1 absolute size-8 right-6 bottom-4 text-primary disabled:text-base-fg-muted z-10 not-[[data-multiline]]:bottom-1/2 not-[[data-multiline]]:translate-y-1/2"
            onclick={submit}
        >
            <PaperAirplane class="size-full" />
        </button>
    </div>
</main>
