import styled from "styled-components"

type Props = {
    className?: string,
    value: number | undefined,
    unit?: string,
    onChange: (value: any) => void,
    max?: number;
    min?: number;
}

const NumericInput = ({ className, value, unit, min, max, onChange }: Props) => {
    if (unit === '%') {
        min = 0;
        max = 100;
    }
    const _onChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        let value = e.target.valueAsNumber;
        if (min !== undefined && value < min) {
            value = min;
        }
        if (max !== undefined && value > max) {
            value = max;
        }
        onChange(value);
    }
    return (
        <div className={className}>
            <input min={min} max={max} type="number" value={value?.toString() ?? ""} onChange={_onChange    } />
            {unit && <span className="unit">{unit}</span>}
        </div>
    )
}

export default styled(NumericInput)`
    display: flex;
    align-items: center;
    width: 100%;
    flex-grow: 1;
    min-width: 180px;
    height: 30px;
    padding: 0 10px;
    border-radius: var(--radius);
    position: relative;
    background: var(--bg);
    border: 1px solid var(--border);
    color: var(--text);
    transition: border-color .12s ease;

    &:hover {
        border-color: var(--border-strong);
    }

    &:focus-within {
        border-color: var(--accent);
    }

    & input {
        all: unset;
        display: flex;
        flex-grow: 1;
        width: 0;
        min-width: 0;
        height: 100%;
        user-select: text;
    }

    & .unit {
        margin-left: 8px;
        font-size: 12px;
        color: var(--text-muted);
    }
`
