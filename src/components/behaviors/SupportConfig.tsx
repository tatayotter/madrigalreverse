import { createSlotBars, SupportConfigModel } from "../../models/BotConfig"
import { FrontendInfoModel } from "../../models/FrontendInfo"

import Modal from '../Modal'
import useModal from '../utils/UseModal'
import YesNoModal from '../YesNoModal'
import SlotBar from "../SlotBar"
import { StopWatchValues, useStopWatch } from "../utils/StopWatch"

import BooleanSlider from '../config/BooleanSlider'
import ConfigLabel from '../config/ConfigLabel'
import ConfigPanel from '../config/ConfigPanel'
import ConfigTable from '../config/ConfigTable'
import ConfigTableRow from '../config/ConfigTableRow'
import styled from "styled-components"
import { DefaultValuesChecker } from "../utils/DefaultValuesChecker"
import NumericInput from "../config/NumericInput"
import TimeInput from "../config/TimeInput"
import { LuSettings } from "react-icons/lu"

type Props = {
    className?: string,
    info: FrontendInfoModel | null,
    config: SupportConfigModel,
    onChange: (config: SupportConfigModel) => void,
    botStopWatch: StopWatchValues | null,
    botState: string,


}

const SupportConfig = ({ className, info, config, onChange, botStopWatch, botState }: Props) => {
    const debugModal = useModal()
    const resetSlotYesNo = useModal(debugModal)
    const onDeathModal = useModal()
    const onAFKModal = useModal()

    const defaultValues = {
        'obstacle_avoidance_cooldown': 2000,
        'interval_between_buffs': 2000,
        'max_main_distance': 100,
    }

    DefaultValuesChecker(config, defaultValues, onChange)

    return (
        <div className={className}>
            <SlotBar botMode="support" config={config} onChange={onChange} />
            <YesNoModal isShowing={resetSlotYesNo.isShown} hide={resetSlotYesNo.close}
                title={<h4>Reset every slot on all bars?</h4>}
                onYes={() => {
                    const newConfig = { ...config, slot_bars: createSlotBars() }
                    onChange(newConfig)
                }} />
            <Modal isShowing={debugModal.isShown} hide={debugModal.close} title={<h4>Settings</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        label={<ConfigLabel name="On death event" helpText="" />}
                        item={<button onClick={onDeathModal.open}>Edit</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="On AFK event" helpText="" />}
                        item={<button onClick={onAFKModal.open}>Edit</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Obstacle avoidance cooldown" helpText="Time before it tries to avoid obstacles, and start movement pattern" />}
                        item={<TimeInput value={config.obstacle_avoidance_cooldown} onChange={value => onChange?.({ ...config, obstacle_avoidance_cooldown: value })} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Interval between buffs" helpText="" />}
                        item={<TimeInput value={config.interval_between_buffs} onChange={value => onChange({ ...config, interval_between_buffs: value })} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Max main distance" helpText="Max distance from main character" />}
                        item={<NumericInput value={config.max_main_distance} onChange={value => onChange?.({ ...config, max_main_distance: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Is it in a party?" helpText="If enabled will buff itself and party leader" />}
                        item={<BooleanSlider value={config.is_in_party ?? false} onChange={value => onChange?.({ ...config, is_in_party: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Duo link" helpText="Share this healer's HP, MP, distance and any giants/violets it sees with the Farming copy of Neuz on this PC (turn on 'Duo with healer' there)" />}
                        item={<BooleanSlider value={config.duo_link ?? false} onChange={value => onChange?.({ ...config, duo_link: value })} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Reset all slots" helpText="" />}
                        item={<button onClick={() => resetSlotYesNo.open()}>Edit</button>}
                    />
                </ConfigTable>
            } />
            <Modal isShowing={onDeathModal.isShown} hide={onDeathModal.close}
                title={<h4>On death behavior</h4>} body={
                    <ConfigTable>
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Disconnect" helpText="If enabled will automatically disconnect the dead character, otherwise we'll try to revive by pressing ENTER" />}
                            item={<BooleanSlider value={config.on_death_disconnect ?? true} onChange={value => onChange?.({ ...config, on_death_disconnect: value })} />}
                        />
                    </ConfigTable>
                } />

            <Modal isShowing={onAFKModal.isShown} hide={onAFKModal.close}
                title={<h4>AFK behavior</h4>} body={
                    <ConfigTable>
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Disconnect" helpText="If enabled will automatically disconnect the AFK character" />}
                            item={<BooleanSlider value={config.on_afk_disconnect ?? false} onChange={value => onChange?.({ ...config, on_afk_disconnect: value })} />}
                        />

                        <ConfigTableRow
                            label={<ConfigLabel name="No action time out" helpText="After this time character will disconnect if AFK Disconnect is enabled" />}
                            item={<TimeInput value={config.afk_timeout} onChange={value => onChange({ ...config, afk_timeout: value })} />}
                        />

                    </ConfigTable>
                } />

            {info && (
                <div className="info">
                    <div className="row">
                        <span className="row__label">Botting time</span>
                        <span className="row__value">{botStopWatch?.toString() ?? "00:00:00:000"}</span>
                    </div>
                    <div className="row row--actions">
                        <button className="btn sm" onClick={debugModal.open}><LuSettings size={13} />Settings</button>
                    </div>
                </div>
            )}
        </div>
    )
}

export default styled(SupportConfig)`
    display: flex;
    flex-direction: column;
    gap: 10px;

    & > :empty {
        display: none;
    }
`
