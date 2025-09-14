import type { ListCollection } from '@zag-js/collection';
import * as passwordInput from '@zag-js/password-input';
import * as select from '@zag-js/select';
import { normalizeProps, useMachine, type PropTypes } from '@zag-js/svelte';

export const createSelect = <T>(props: Omit<select.Props<T>, 'collection'> & {
    collection: ListCollection<T>
}): select.Api<PropTypes, T> => {
    const service = useMachine(select.machine, props as unknown as select.Props<T>);
    const api = $derived(select.connect<PropTypes, T>(service, normalizeProps));
    return {
        get focused() {
            return api.focused;
        },
        get open() {
            return api.open;
        },
        get empty() {
            return api.empty;
        },
        get highlightedValue() {
            return api.highlightedValue;
        },
        get highlightedItem() {
            return api.highlightedItem;
        },
        setHighlightValue(value: string) {
            return api.setHighlightValue(value);
        },
        clearHighlightValue() {
            return api.clearHighlightValue();
        },
        get selectedItems() {
            return api.selectedItems;
        },
        get hasSelectedItems() {
            return api.hasSelectedItems;
        },
        get value() {
            return api.value;
        },
        get valueAsString() {
            return api.valueAsString;
        },
        selectValue(value: string) {
            return api.selectValue(value);
        },
        selectAll() {
            return api.selectAll();
        },
        setValue(value: string[]) {
            return api.setValue(value);
        },
        clearValue(value?: string) {
            return api.clearValue(value);
        },
        focus() {
            return api.focus();
        },
        getItemState(props) {
            return api.getItemState(props);
        },
        setOpen(open) {
            return api.setOpen(open);
        },
        get collection() {
            return api.collection;
        },
        reposition(options?) {
            return api.reposition(options);
        },
        get multiple() {
            return api.multiple;
        },
        get disabled() {
            return api.disabled;
        },
        getRootProps() {
            return api.getRootProps();
        },
        getLabelProps() {
            return api.getLabelProps();
        },
        getControlProps() {
            return api.getControlProps();
        },
        getTriggerProps() {
            return api.getTriggerProps();
        },
        getIndicatorProps() {
            return api.getIndicatorProps();
        },
        getClearTriggerProps() {
            return api.getClearTriggerProps();
        },
        getValueTextProps() {
            return api.getValueTextProps();
        },
        getPositionerProps() {
            return api.getPositionerProps();
        },
        getContentProps() {
            return api.getContentProps();
        },
        getListProps() {
            return api.getListProps();
        },
        getItemProps(props) {
            return api.getItemProps(props);
        },
        getItemTextProps(props) {
            return api.getItemTextProps(props);
        },
        getItemIndicatorProps(props) {
            return api.getItemIndicatorProps(props);
        },
        getItemGroupProps(props) {
            return api.getItemGroupProps(props);
        },
        getItemGroupLabelProps(props) {
            return api.getItemGroupLabelProps(props);
        },
        getHiddenSelectProps() {
            return api.getHiddenSelectProps();
        },
    };
}

export const createPasswordInput = (props: passwordInput.Props): passwordInput.Api => {
    const service = useMachine<passwordInput.PasswordInputSchema>(passwordInput.machine, props);
    const api = $derived(passwordInput.connect(service, normalizeProps));
    return {
        get visible() {
            return api.visible;
        },
        get disabled() {
            return api.disabled;
        },
        get invalid() {
            return api.invalid;
        },
        focus() {
            return api.focus();
        },
        setVisible(value: boolean) {
            return api.setVisible(value);
        },
        toggleVisible() {
            return api.toggleVisible();
        },
        getRootProps() {
            return api.getRootProps();
        },
        getLabelProps() {
            return api.getLabelProps();
        },
        getInputProps() {
            return api.getInputProps();
        },
        getVisibilityTriggerProps() {
            return api.getVisibilityTriggerProps();
        },
        getIndicatorProps() {
            return api.getIndicatorProps();
        },
        getControlProps() {
            return api.getControlProps();
        },
    };
}