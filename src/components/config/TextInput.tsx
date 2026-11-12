import styled from "styled-components"

type Props = {
    className?: string,
    value: string | undefined,
    unit?: string,
    min?: number,
    max?: number,
    onChange: (value: string) => void,
    condition?: (value: any) => boolean,
}

const TextInput = ({ className, value, unit,min = 0, max, onChange, condition }: Props) => {
    return (
        <div className={className}>
            <input type="text" value={value?.toString() ?? ""} onChange={(e)=>onChange(e.target.value)} />
            {unit && <span className="unit">{unit}</span>}
        </div>
    )
}

export default styled(TextInput)`
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
