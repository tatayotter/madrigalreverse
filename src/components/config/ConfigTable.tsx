import React from 'react'
import styled from 'styled-components'

type Props = React.PropsWithChildren<{
    className?: string,
}>

const ConfigTable = ({ className, children }: Props) => {
    return (
        <table className={className}>
            <tbody>
                {React.Children.map(children, child => <tr>{child}</tr>)}
            </tbody>
        </table>
    )
}

export default styled(ConfigTable)`
    width: 100%;
    border: none;
    border-collapse: collapse;
    color: var(--text);

    & > tbody > tr {
        border-bottom: 1px solid var(--border);
    }

    & > tbody > tr:last-child {
        border-bottom: none;
    }
`
