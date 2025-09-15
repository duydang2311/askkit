import clsx from 'clsx';

export const button = ({
    variant = 'base',
    filled,
    border,
}: { variant?: 'base' | 'primary'; filled?: boolean; border?: boolean } = {}) => {
    return clsx(
        'c-button',
        `c-button--${variant}`,
        filled && 'c-button--filled',
        border && 'c-button--border'
    );
};
