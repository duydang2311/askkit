<script lang="ts" generics="T extends CollectionItem">
    import type { CollectionItem, ListCollection } from '@zag-js/collection';
    import * as select from '@zag-js/select';
    import { normalizeProps, useMachine, type PropTypes } from '@zag-js/svelte';
    import type { Snippet } from 'svelte';

    interface Props extends Omit<select.Props<T>, 'id' | 'value' | 'collection'> {
        children: Snippet<[select.Api<PropTypes, T>]>;
        collection: ListCollection<T> | (() => ListCollection<T>);
        value?: string[] | (() => string[] | undefined);
    }

    let { collection, value, children, ...props }: Props = $props();
    const id = $props.id();
    const service = useMachine(select.machine, {
        id,
        // @ts-ignore
        get collection() {
            return collection instanceof Function ? collection() : collection;
        },
        get value() {
            return value instanceof Function ? value() : value;
        },
        ...props,
    });
    const selectApi = $derived(select.connect<PropTypes, T>(service, normalizeProps));

    export const api = () => {
        return selectApi;
    };
</script>

{@render children(selectApi)}
