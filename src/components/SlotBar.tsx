import styled from 'styled-components'

import Slot from './Slot'

import { createSlotBars, FarmingConfigModel, SlotModel, SupportConfigModel } from '../models/BotConfig'
import SlotModal from './SlotModal'
import useModal from './utils/UseModal'
import { useState } from 'react'
import { useKeyPress } from './utils/KeyboardHotkeys'
import { LuChevronDown, LuChevronUp } from 'react-icons/lu'

type Props = {
    className?: string,
    config: FarmingConfigModel | SupportConfigModel,
    onChange: (config: SupportConfigModel | FarmingConfigModel) => void,
    botMode: string
}

const SlotBar = ({ className, config, botMode, onChange }: Props) => {
    const { isShown, toggle } = useModal();
    const [currentSlotId, setCurrentSlotId] = useState(-1)
    const [currentBarIndex, setCurrentBarIndex] = useState(0)
    const toogleSlot = (id: number, event: any,targetSlot: SlotModel ) => {
        if(event.shiftKey){
            targetSlot.slot_enabled = !targetSlot.slot_enabled;
            handleSlotChange(currentBarIndex, id, targetSlot);
        }else{
            setCurrentSlotId(id)
            toggle()
        }
    }
    let slots = config.slot_bars ?? createSlotBars()

    // Bind Neuz slotbars with F1-F9 just like in game
    useKeyPress(["F1","F2","F3","F4","F5","F6","F7","F8","F9",], (event: { key: string }) => {
        setCurrentBarIndex(parseInt(event.key.replace("F","")) - 1)
    })

    const handleSlotChange = (slot_bar_index:number, slot_index:number, slot: SlotModel) => {
        const newConfig = { ...config, slot_bars: config.slot_bars ?? createSlotBars() }
        newConfig.slot_bars[slot_bar_index].slots[slot_index] = slot
        onChange(newConfig)
    }

    return (
        <>
            <SlotModal botMode={botMode} isShowing={isShown} hide={toggle} index={currentSlotId} slot={slots[currentBarIndex].slots[currentSlotId]} onChange={handleSlotChange} barIndex={currentBarIndex} indexName={currentSlotId +""}/>
            <div className={className}>

                <div className="slots">
                    {slots[currentBarIndex].slots.map((slot, index) =>  (
                        <Slot key={index} type={slot.slot_type} index={index} toggleSlotModal={(event,targetSlot) => toogleSlot(index,event,targetSlot)} indexName={index +""} slot={slots[currentBarIndex].slots[index]} />

                    ))}
                </div>
                <div className="barPicker" title="Slot bar (F1-F9 also switch bars)">
                    <button className={`changeBtn ${currentBarIndex === 8 ? 'grayedOutBtn' : ''}`} onClick={() => {
                        if (currentBarIndex != 8) {
                            setCurrentBarIndex(currentBarIndex+1)
                        }
                    }}>
                        <LuChevronUp size={12} />
                    </button>
                    <div className="slotIndexDisplay">F{currentBarIndex + 1}</div>
                    <button className={`changeBtn ${currentBarIndex === 0 ? 'grayedOutBtn' : ''}`} onClick={() => {
                        if (currentBarIndex != 0) {
                            setCurrentBarIndex(currentBarIndex-1)
                        }
                    }}>
                        <LuChevronDown size={12} />
                    </button>
                </div>
            </div>
        </>
    )
}

export default styled(SlotBar)`
    display: flex;
    align-items: stretch;
    gap: 8px;
    padding: 0 8px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);

    & .slots {
        flex: 1;
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: center;
        gap: 4px;
    }

    & .barPicker {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 2px;
        padding-left: 8px;
        border-left: 1px solid var(--border);
    }

    & .slotIndexDisplay {
        font-size: 12px;
        font-weight: 600;
        color: var(--text);
        font-variant-numeric: tabular-nums;
    }

    & .changeBtn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 22px;
        height: 18px;
        padding: 0;
        border: none;
        border-radius: 4px;
        background: transparent;
        color: var(--text-muted);
        cursor: pointer;

        &:hover {
            background: var(--surface-2);
            color: var(--text);
        }
    }

    & .grayedOutBtn {
        opacity: .3;
        pointer-events: none;
    }
`
