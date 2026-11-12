import styled from "styled-components"
import { useState } from 'react';

type Props = {
    className?: string,
    messages?: string[],
    onChange: (value: string[]) => void,
}

const StringList = ({ className, messages, onChange }: Props) => {
    const [messageBuffer, setMessageBuffer] = useState<string[]>(messages ?? []);
    const getValue = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
        return e.target.value.trim().length === 0
            ? []
            : e.target.value.split('\n').map(line => line)
    }
    return (
        <div className={className}>
            <textarea value={messageBuffer?.join("\n") ?? ""} onChange={e => setMessageBuffer(getValue(e))} />
            <button className="btn sm primary" onClick={() => onChange?.(messageBuffer)}>Apply</button>
        </div>
    )
}

export default styled(StringList)`
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;

    & textarea {
        width: 100%;
        min-height: 96px;
        padding: 8px 10px;
        resize: vertical;
        border-radius: var(--radius);
        background: var(--bg);
        border: 1px solid var(--border);
        color: var(--text);
        outline: none;
        user-select: text;

        &:focus {
            border-color: var(--accent);
        }
    }

    & .btn {
        align-self: flex-end;
    }
`
