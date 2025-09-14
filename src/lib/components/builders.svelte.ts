import * as passwordInput from '@zag-js/password-input';
import { normalizeProps, useMachine } from '@zag-js/svelte';

export const createPasswordInput = (props: passwordInput.Props) => {
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