import styled from "styled-components"

type Props = {
    className?: string,
    value: boolean,
    onChange: (value: boolean) => void,
}

const BooleanSlider = ({ className, value, onChange }: Props) => {
    return (
        <div className={className} role="switch" aria-checked={value} data-active={value} onClick={() => onChange(!value)}>
            <div className="symbol" />
        </div>
    )
}

export default styled(BooleanSlider)`
    position: relative;
    flex-shrink: 0;
    width: 34px;
    height: 20px;
    border-radius: 999px;
    background: var(--surface-3);
    border: 1px solid var(--border-strong);
    cursor: pointer;
    transition: background-color .15s ease, border-color .15s ease;

    & .symbol {
        position: absolute;
        top: 2px;
        left: 2px;
        width: 14px;
        height: 14px;
        border-radius: 50%;
        background: var(--text-muted);
        transition: transform .15s ease, background-color .15s ease;
    }

    &[data-active=true] {
        background: var(--accent);
        border-color: var(--accent);

        & .symbol {
            transform: translateX(14px);
            background: #fff;
        }
    }

    &:hover {
        border-color: var(--accent-hover);
    }
`
