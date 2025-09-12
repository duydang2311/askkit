export interface AppError {
    kind: string;
    message?: string;
}

export const isAppError = (error: unknown): error is AppError => {
    if (typeof error === 'object' && error !== null) {
        return 'kind' in error && typeof (error as AppError).kind === 'string';
    }
    return false;
}