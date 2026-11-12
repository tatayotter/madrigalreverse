import { useState } from "react";
import styled from "styled-components"
import { StopWatchValues } from "../utils/StopWatch";

type Props = {
    className?: string,
    value: number | undefined,
    onChange: (value: any) => void,

}

const TimeInput = ({ className, value, onChange }: Props) => {
    let stopWatch = new StopWatchValues(value ?? 0)
    const [newValue, setNewValue] = useState(stopWatch.toString())

    return (
        <div className={className}>
            <input type="text" value={newValue?.toString() ?? ""}
                onChange={(e)=>{
                    stopWatch.fromString(e.target.value)
                        if (stopWatch.timer > 0 || e.target.value == stopWatch.toString() ) {
                        onChange(stopWatch.timer)
                    }
                    setNewValue(e.target.value)
                }}
            />
            {!stopWatch.validStringFormat(newValue) ?
                <span className="unit invalid" title="Format: hh:mm:ss:ms. Click to restore the last valid value" onClick={() => {setNewValue(stopWatch.toString())}}>Invalid</span> : <span className="unit">hh:mm:ss:ms</span>
            }
        </div>
    )
}

export default styled(TimeInput)`
    display: flex;
    align-items: center;
    width: 100%;
    flex-grow: 1;
    min-width: 200px;
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

    & .unit.invalid {
        color: var(--danger);
        cursor: pointer;
    }
`
