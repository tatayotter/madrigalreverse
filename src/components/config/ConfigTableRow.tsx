import styled from 'styled-components'

type Props = {
    label: React.ReactNode,
    item: React.ReactNode,
    layout?: "h" | "v",
    disabled?: boolean,
}

type ItemProps = {
    disabled: boolean,
}

const Label = styled.td`
    width: 100%;
    padding: 8px 0;
    vertical-align: middle;
`

const FlexLabel = styled.td`
    width: 100%;
    display: flex;
    justify-content: flex-start;
    align-items: center;
    padding: 8px 0 4px;
`

const Item = styled.td<ItemProps>`
    padding: 8px 0 8px 12px;
    vertical-align: middle;
    white-space: nowrap;

    & > .cell {
        display: flex;
        justify-content: flex-end;
        align-items: center;
    }


    &[disabled] {
        opacity: .5;
        filter: grayscale(100%);
        pointer-events: none;
    }
`
const ConfigTableRow = ({ label, item, layout = "h", disabled = false }: Props) => {
    return (
        <>
            {layout === "h" && (
                <Label>{label}</Label>
            )}
            {layout === "v" && (
                <FlexLabel>{label}</FlexLabel>
            )}
            <Item disabled={disabled}><div className="cell">{item}</div></Item>
        </>
    )
}

export default styled(ConfigTableRow)`
`
