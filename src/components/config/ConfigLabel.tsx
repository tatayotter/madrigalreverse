import { useRecoilState } from 'recoil'
import styled from 'styled-components'

import { activeHelpState } from '../../state/atoms'

type Props = {
    className?: string,
    name: string,
    helpText?: string,
}

const ConfigLabel = ( { className, name, helpText }: Props) => {
    const [activeHelpId, setActiveHelpId] = useRecoilState(activeHelpState)
    const showHelp = activeHelpId === name

    return (
        <div className={className}>
            <div className="inner">
                <span className="name">{name}</span>
                {helpText && (
                    <div className="help" onClick={() => setActiveHelpId(showHelp ? null : name)}>
                        <abbr title={helpText} className="help__symbol">?</abbr>
                    </div>
                )}
            </div>
            {helpText && showHelp && (
                <div className="help-content">
                    {helpText}
                </div>
            )}
        </div>
    )
}

export default styled(ConfigLabel)`
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 6px;
    color: var(--text);

    & .inner {
        display: flex;
        align-items: center;
        justify-content: flex-start;
        gap: 6px;
        margin-right: 12px;
    }

    & .name {
        color: var(--text);
    }

    & .help {
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
        width: 15px;
        height: 15px;
        border-radius: 50%;
        border: 1px solid var(--border-strong);
        color: var(--text-muted);
        cursor: help;

        &:hover {
            border-color: var(--accent);
            color: var(--accent);
        }

        &__symbol {
            font-size: 10px;
            font-weight: 700;
            text-decoration: none;
        }
    }

    & .help-content {
        padding: 6px 10px;
        border-left: 2px solid var(--accent);
        border-radius: 0 var(--radius) var(--radius) 0;
        background: var(--accent-soft);
        color: var(--text-muted);
        font-size: 12px;
        line-height: 1.45;
        word-wrap: break-word;
    }
`
