import styled from "styled-components"

type Props = {
    className?: string,
    value: number[],
    onChange: (value: number[]) => void,
}

const NumericInput = ({ className, value,  onChange }: Props) => {
    return (
        <div className={className}>
            <span className="unit first">R</span>
            <input min={0} max={255} type="number" value={value[0]} onChange={(e)=>{value[0] = e.target.valueAsNumber;onChange(value)}} />
            <span className="unit">G</span>
            <input min={0} max={255} type="number" value={value[1]} onChange={(e)=>{value[1] = e.target.valueAsNumber;onChange(value)}} />
            <span className="unit">B</span>
            <input min={0} max={255} type="number" value={value[2]} onChange={(e)=>{value[2] = e.target.valueAsNumber;onChange(value)}} />
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

    & .unit {
        margin: 0 6px 0 0;
        padding-left: 8px;
        border-left: 1px solid var(--border);
    }

    & .unit.first {
        padding-left: 0;
        border-left: none;
    }
`
