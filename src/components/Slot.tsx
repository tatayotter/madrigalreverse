import styled from 'styled-components'

import { SlotModel, SlotType, slotTypes, SLOT_SIZE_PX, translateDesc, translateType } from '../models/BotConfig'

type Props = {
    className?: string,
    type: SlotType,
    index: number,
    indexName: string,
    onChange?: (type: SlotType) => void,
    toggleSlotModal: (event:any,targetSlot: SlotModel) => void,
    slot: SlotModel,
}

const Slot = ({ className, type = 'Unused', index, onChange, toggleSlotModal, indexName,slot }: Props) => {
    const handleChange = () => {
        const nextType: SlotType = slotTypes[(slotTypes.indexOf(type) + 1) % slotTypes.length];
        onChange?.(nextType)
    }
    const symbolOrIcon = translateType(type)
    const useIcon = symbolOrIcon.startsWith('data:') || symbolOrIcon.includes('static');
    const slotState = !slot || slot.slot_type === "Unused" ? "unused" : slot.slot_enabled === true ? "enabled" : "disabled"
    return (

        <div className={className} data-state={slotState} title={slot && slot.slot_type !== "Unused" ? `${translateDesc(type)[1]} (${slot.slot_enabled ? "on" : "off"}). Shift-click to toggle` : "Empty slot"} onClick={(e) => toggleSlotModal(e,slot)} >
            <div className="index">{indexName}</div>
            <div className='slot' onClick={handleChange}>
                {useIcon && (
                    <img className="type" src={symbolOrIcon} alt="Slot icon" />
                )}
                {!useIcon && (
                    <div className="type">{symbolOrIcon}</div>
                )}
                <div className="desc">{translateDesc(type)[0]}</div>
            </div>
        </div>
    )
}

export default styled(Slot)`
    display: flex;
    align-items: center;
    justify-content: center;
    width: ${SLOT_SIZE_PX}px;
    height: ${SLOT_SIZE_PX}px;
    background-color: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    position: relative;
    margin: 14px 0;
    cursor: pointer;
    transition: border-color .12s ease, background-color .12s ease;

    &:first-child {
        order: 1 !important;
    }

    &[data-state=enabled] {
        border-color: rgba(47, 191, 113, .7);
        box-shadow: inset 0 0 0 1px rgba(47, 191, 113, .15);
    }

    &[data-state=disabled] {
        border-color: rgba(229, 72, 77, .55);

        & .slot {
            opacity: .45;
        }
    }

    &:hover {
        background-color: var(--surface-2);
        border-color: var(--accent);
    }

    & .slot {
        height: 100%;
        width: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    & .index {
        position: absolute;
        top: -15px;
        width: 100%;
        text-align: center;
        font-size: 10px;
        color: var(--text-faint);
        font-variant-numeric: tabular-nums;
    }

    & .desc {
        position: absolute;
        bottom: -15px;
        width: 100%;
        text-align: center;
        font-size: 10px;
        color: var(--text-muted);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    & div.type {
        color: var(--text);
        font-size: 18px;
        line-height: 1;
    }

    & img.type {
        width: 100%;
        height: 100%;
        object-fit: contain;
        padding: 4px;
        border-radius: var(--radius);
    }
`
