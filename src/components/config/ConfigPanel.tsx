import styled from 'styled-components'

type Props = React.PropsWithChildren<{
    className?: string,
}>

const ConfigPanel = ({ className, children }: Props) => {
    return (
        <div className={className}>
            {children}
        </div>
    )
}

export default styled(ConfigPanel)`
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    color: var(--text);
    overflow: auto;
`
