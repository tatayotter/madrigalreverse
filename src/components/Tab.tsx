import styled from 'styled-components'
import { IconType } from 'react-icons/lib'

import { ModeModel } from '../models/BotConfig'

type Props = {
    className?: string,
    label: string,
    icon?: IconType,
    mode: ModeModel,
    activeMode?: ModeModel,
    onSelect?: (mode: ModeModel) => void,
}

const Tab = ({ className, label, icon: Icon, mode, activeMode, onSelect }: Props) => {
    const active = activeMode === mode
    return (
        <button
            type="button"
            className={className}
            data-active={active}
            onClick={() => onSelect?.(mode)}
        >
            {Icon && <Icon size={14} />}
            <span>{label}</span>
        </button>
    )
}

export default styled(Tab)`
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 30px;
    border: none;
    border-radius: 5px;
    background: transparent;
    color: var(--text-muted);
    font-weight: 500;
    cursor: pointer;
    transition: background-color .12s ease, color .12s ease;

    &:hover {
        color: var(--text);
    }

    &[data-active=true] {
        background: var(--surface-3);
        color: var(--text);
        box-shadow: 0 1px 2px rgba(0, 0, 0, .35);
    }
`
