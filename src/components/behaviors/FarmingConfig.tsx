import styled from 'styled-components'

import BooleanSlider from '../config/BooleanSlider'
import ConfigLabel from '../config/ConfigLabel'
import ConfigTable from '../config/ConfigTable'
import ConfigTableRow from '../config/ConfigTableRow'
import NumericInput from '../config/NumericInput'
import ColorSelector from '../config/ColorSelector'

import SlotBar from '../SlotBar'
import { createSlotBars, FarmingConfigModel } from '../../models/BotConfig'
import { useEffect, useRef, useState } from 'react'
import { FrontendInfoModel } from '../../models/FrontendInfo'
import Modal from '../Modal'
import useModal from '../utils/UseModal'
import YesNoModal from '../YesNoModal'
import { MsFormat, StopWatchValues } from '../utils/StopWatch'
import { DefaultValuesChecker } from '../utils/DefaultValuesChecker'
import TimeInput from '../config/TimeInput'
import StringList from '../config/StringList'
import { invoke } from '@tauri-apps/api'

type Props = {
    className?: string,
    info: FrontendInfoModel | null,
    config: FarmingConfigModel,
    onChange: (config: FarmingConfigModel) => void,
    botStopWatch: StopWatchValues | null,
    botState: string,
}

const FarmingConfig = ({ className, info, config, onChange, botStopWatch, botState }: Props) => {
    const statsModal = useModal()
    const debugModal = useModal()
    const debugWarningModal = useModal(debugModal)
    const onDeathModal = useModal(debugModal)
    const onAFKModal = useModal()

    const mobsNameDebugModal = useModal(debugModal)
    const mobsColorsDebugModal = useModal(mobsNameDebugModal)
    const resetSlotYesNo = useModal(debugModal)
    const obstacleAvoidanceDebugModal = useModal(debugModal)
    const fleeModal = useModal(debugModal)
    const namesModal = useModal(debugModal)
    const duoModal = useModal(debugModal)

    const selectedMobType = useRef(0)

    const defaultValues = {
        'passive_mobs_colors': [234, 234, 149],
        'passive_tolerence': 5,
        'aggressive_mobs_colors': [179, 23, 23],
        'aggressive_tolerence': 10,
        'obstacle_avoidance_cooldown': 3000,
        'obstacle_avoidance_max_try': 3,
        'min_mobs_name_width': 15,
        'max_mobs_name_width': 180,
        'circle_pattern_rotation_duration': 30,
        'min_hp_attack': 30,
        'prevent_already_attacked': true,
        'prioritize_aggro': true,
        'interval_between_buffs': 2000,
        'aoe_farming': 1,
    }

    DefaultValuesChecker(config, defaultValues, onChange)

    const colorsRefResetter = [
        () => onChange({...config, ...{passive_mobs_colors: defaultValues['passive_mobs_colors'], passive_tolerence: defaultValues['passive_tolerence']} }),
        () => onChange({...config, ...{aggressive_mobs_colors: defaultValues['aggressive_mobs_colors'], aggressive_tolerence: defaultValues['aggressive_tolerence']} })
    ]

    const [debugMode, setDebugMode] = useState(false);

    // Screenshot recorder: 60 frames, one per second
    const RECORD_FRAMES = 60
    const [recordingLeft, setRecordingLeft] = useState(0)
    useEffect(() => {
        if (recordingLeft <= 0) return
        const timer = setTimeout(() => setRecordingLeft(recordingLeft - 1), 1000)
        return () => clearTimeout(timer)
    }, [recordingLeft])
    const toggleRecording = () => {
        if (recordingLeft > 0) {
            invoke('stop_recording').then(() => setRecordingLeft(0))
        } else {
            invoke('start_recording', { frames: RECORD_FRAMES, intervalMs: 1000 })
                .then(dir => { if (dir) setRecordingLeft(RECORD_FRAMES) })
        }
    }

    useEffect(() => {
        if (debugMode) {
            debugWarningModal.open()
        }
    }, [debugMode])
    // StopWatchs
    const searchMobStopWatch = MsFormat(info?.last_search_duration ?? 0),
    fightStopWatch = MsFormat(info?.last_fight_duration ?? 0)

    const globalKPM = ((info?.enemy_kill_count?? 0) / Math.round(Number(botStopWatch?.timer ?? 0) / 60000)).toFixed(2)
    const globalKPH = (Number(globalKPM) * 60).toFixed(2)

    return (
        <div className={className}>
            <SlotBar botMode="farming" config={config} onChange={onChange} />
            {/* DEBUG */}
            <YesNoModal isShowing={resetSlotYesNo.isShown} hide={resetSlotYesNo.close}
                title={<h4>Confirm slot reset this action is irreversible</h4>}
                onYes={() => {onChange({ ...config, slot_bars: createSlotBars() })}}/>
            <Modal isShowing={debugModal.isShown} hide={debugModal.close} title={<h4>Settings</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        label={<ConfigLabel name="Mobs detection" helpText="" />}
                        item={<button onClick={mobsNameDebugModal.open}>⚙️</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Avoidances" helpText="" />}
                        item={<button onClick={obstacleAvoidanceDebugModal.open}>⚙️</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Duo with healer" helpText="Coordinate with a second Neuz running Support mode on this PC" />}
                        item={<button onClick={duoModal.open}>⚙️</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Monster names" helpText="Read monster names from the screen to skip giants and pick which monsters to attack" />}
                        item={<button onClick={namesModal.open}>⚙️</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Giant & violet flee" helpText="Run away when a violet or giant gets close" />}
                        item={<button onClick={fleeModal.open}>⚙️</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="On death event" helpText="" />}
                        item={<button onClick={onDeathModal.open}>⚙️</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Interval between buffs" helpText="" />}
                        item={<TimeInput value={config.interval_between_buffs} onChange={value => onChange({...config, interval_between_buffs: value})} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Debug" />}
                        item={<BooleanSlider value={debugMode} onChange={value => setDebugMode(value)} />}
                    />

                    <ConfigTableRow
                        label={<ConfigLabel name="Record screenshots" helpText="Saves one game frame per second for 60 seconds, plus what the bot detected in each, so problems can be checked offline. The bot must be engaged (running)." />}
                        item={<button onClick={toggleRecording}>{recordingLeft > 0 ? `⏹ Stop (${recordingLeft}s)` : '⏺ Record'}</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Recordings folder" helpText="Zip a recording folder and send it to get detection fixed" />}
                        item={<button onClick={() => invoke('open_recordings_folder')}>📁</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Reset all slots" helpText="" />}
                        item={<button onClick={resetSlotYesNo.open}>⚙️</button>}
                    />
                </ConfigTable>
            }/>
            <Modal isShowing={mobsColorsDebugModal.isShown} hide={mobsColorsDebugModal.close} title={(selectedMobType.current === 0)? <h4>Passive mob detection</h4> : <h4>Aggressive mob detection</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Colors" helpText="Monster's name color reference. Edit these values if you are sure what you are doing." />}
                        item={<ColorSelector value={(selectedMobType.current === 0)? config.passive_mobs_colors ?? [] : config.aggressive_mobs_colors ?? []} onChange={value => onChange?.((selectedMobType.current === 0)?{ ...config, passive_mobs_colors: value}: { ...config, aggressive_mobs_colors: value})} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Tolerence" helpText="Monster's name color tolerence. Edit these values if you are sure what you are doing." />}
                        item={<NumericInput min={0} max={255} unit="#" value={(selectedMobType.current === 0)? config.passive_tolerence : config.aggressive_tolerence} onChange={value => onChange?.((selectedMobType.current === 0)? { ...config, passive_tolerence: value } : { ...config, aggressive_tolerence: value })} />}
                    />
                    <ConfigTableRow
                    layout="v"
                        label={<ConfigLabel name="" helpText="" />}
                        item={<button onClick={()=>colorsRefResetter[selectedMobType.current]()}>Reset</button>}
                    />
                </ConfigTable>
            }/>
            <Modal isShowing={obstacleAvoidanceDebugModal.isShown} hide={obstacleAvoidanceDebugModal.close} title={<h4>Avoidances</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Avoid attacked monster" helpText="Anti-killsteal: skip monsters that are already hurt when you target them, unless something is hitting you. Disable when you play in a party" />}
                        item={<BooleanSlider value={config.prevent_already_attacked ?? true} onChange={value => onChange?.({ ...config, prevent_already_attacked: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Obstacle avoidance cooldown" helpText="Time before we try to move or escape if monster's HP doesn't change" />}
                        item={<TimeInput value={config.obstacle_avoidance_cooldown} onChange={value => onChange({...config, obstacle_avoidance_cooldown: value})} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Obstacle avoidance max try" helpText="After this number of try it'll abort attack and search for another target" />}
                        item={<NumericInput unit='#' value={config.obstacle_avoidance_max_try} onChange={value => onChange({...config, obstacle_avoidance_max_try: value})} />}
                    />
                </ConfigTable>
            }/>
            <Modal isShowing={fleeModal.isShown} hide={fleeModal.close} title={<h4>Giant & violet flee</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Enabled" helpText="When a violet (or giant, if a color is set below) is close, drop the target, turn away and run" />}
                        item={<BooleanSlider value={config.giant_flee ?? false} onChange={value => onChange?.({ ...config, giant_flee: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Flee radius" helpText="Distance from your character (in screen pixels) at which the bot runs away. Default: 250" />}
                        item={<NumericInput unit='px' value={config.flee_radius ?? 250} onChange={value => onChange({...config, flee_radius: value})} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Giant name color" helpText="Optional. Name color of giants to flee from. Leave empty to only flee from violets." />}
                        item={<ColorSelector value={config.giant_mobs_colors ?? []} onChange={value => onChange?.({ ...config, giant_mobs_colors: [value[0], value[1], value[2]] })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Giant color tolerence" helpText="How close a pixel must be to the giant color. Default: 10" />}
                        item={<NumericInput min={0} max={255} unit="#" value={config.giant_tolerence ?? 10} onChange={value => onChange?.({ ...config, giant_tolerence: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="" helpText="" />}
                        item={<button onClick={() => onChange?.({ ...config, giant_mobs_colors: undefined })}>Clear giant color</button>}
                    />
                </ConfigTable>
            }/>
            <Modal isShowing={duoModal.isShown} hide={duoModal.close} title={<h4>Duo with healer</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Wait for healer" helpText="Run a second Neuz in Support mode with 'Duo link' on. Between fights, the farmer waits while the healer is dead, low on HP/MP or too far away. If Giant & violet flee is on, the farmer also runs when the healer sees a giant or violet." />}
                        item={<BooleanSlider value={config.duo_wait_for_healer ?? false} onChange={value => onChange?.({ ...config, duo_wait_for_healer: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Protect healer" helpText="When the healer is being hit, drop passive targets and go for the aggressive (red) monsters. Default: on" />}
                        item={<BooleanSlider value={config.duo_protect_healer ?? true} onChange={value => onChange?.({ ...config, duo_protect_healer: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Board up when healer dies" helpText="Drop the fight and get on the board (monsters don't attack you while mounted), then wait until the healer is back. Put your board in a slot set to 'Board'. Default: on" />}
                        item={<BooleanSlider value={config.duo_escape_on_healer_death ?? true} onChange={value => onChange?.({ ...config, duo_escape_on_healer_death: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Healer min HP" helpText="Wait while the healer's HP is below this. Default: 40" />}
                        item={<NumericInput unit='%' value={config.duo_healer_min_hp ?? 40} onChange={value => onChange({...config, duo_healer_min_hp: value})} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Healer min MP" helpText="Wait while the healer's MP is below this. Default: 20" />}
                        item={<NumericInput unit='%' value={config.duo_healer_min_mp ?? 20} onChange={value => onChange({...config, duo_healer_min_mp: value})} />}
                    />
                </ConfigTable>
            }/>
            <Modal isShowing={namesModal.isShown} hide={namesModal.close} title={<h4>Monster names</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Read monster names" helpText="Windows only. Reads name tags with the OCR built into Windows and looks them up in the Flyff Universe monster list. Giants, violets and bosses are never attacked, and are fled from when Giant & violet flee is on." />}
                        item={<BooleanSlider value={config.name_reading ?? false} onChange={value => onChange?.({ ...config, name_reading: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Only attack these" helpText="One monster name per line, then press Apply. Leave empty to attack everything." />}
                        item={<StringList messages={config.monster_whitelist} onChange={value => onChange?.({ ...config, monster_whitelist: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Never attack these" helpText="One monster name per line, then press Apply." />}
                        item={<StringList messages={config.monster_blacklist} onChange={value => onChange?.({ ...config, monster_blacklist: value })} />}
                    />
                </ConfigTable>
            }/>
            <Modal isShowing={mobsNameDebugModal.isShown} hide={mobsNameDebugModal.close} title={<h4>Mobs detection</h4>} body={
                <ConfigTable>
                    { debugMode &&
                    <>
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Passive mob detection" helpText="" />}
                            item={<button onClick={() => {
                                selectedMobType.current = 0
                                mobsColorsDebugModal.open()
                            }}>⚙️</button>}
                        />
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Agressive mob detection" helpText="" />}
                            item={<button onClick={() => {
                                selectedMobType.current = 1
                                mobsColorsDebugModal.open()
                            }}>⚙️</button>}
                        />
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Min mobs name width" helpText="" />}
                            item={<NumericInput unit='px' value={config.min_mobs_name_width} onChange={value => onChange({...config, min_mobs_name_width: value})} />}
                        />
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Max mobs name width" helpText="" />}
                            item={<NumericInput unit='px' value={config.max_mobs_name_width} onChange={value => onChange({...config, max_mobs_name_width: value})} />}
                        />
                    </> }
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Circle pattern duration" helpText="The bot will try to move in a circle pattern to find targets. Value of 0 will stay in place. Lower the value to increase circle size. Default : 30" />}
                        item={<NumericInput value={config.circle_pattern_rotation_duration} onChange={value => onChange?.({ ...config, circle_pattern_rotation_duration: value })} />}
                    />

                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Min HP percent to attack" helpText="Minimum required HP value to attack a monster (only for passive ones)" />}
                        item={<NumericInput unit='%' value={config.min_hp_attack} onChange={value => onChange({...config, min_hp_attack: value})} />}
                    />


                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Max number of mobs to attack" helpText="Max number of mobs to attack at the same time" />}
                        item={<NumericInput value={config.aoe_farming} onChange={value => onChange?.({ ...config, aoe_farming: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Prioritize Aggro" helpText="When selecting a target, give priorities to aggro" />}
                        item={<BooleanSlider value={config.prioritize_aggro ?? true} onChange={value => onChange?.({ ...config, prioritize_aggro: value })} />}
                    />

                </ConfigTable>
            }/>
            <Modal isShowing={debugWarningModal.isShown} hide={debugWarningModal.close}
            title={<h4>WARNING - Debug mode </h4>} body={
                <div className='modalCenteredContent'>
                    <h1>Warning !</h1>
                    <h2>For advanced users ONLY ! If something went wrong after you set something don't need to cry just reset your profile</h2>
                </div>
            }/>
            {/* DEBUG END */}
            <Modal isShowing={onDeathModal.isShown} hide={onDeathModal.close}
            title={<h4>Auto disconnect</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        label={<ConfigLabel name="No mob time out" helpText="After this time character will disconnect if no mobs were found" />}
                        item={<TimeInput value={config.mobs_timeout} onChange={value => onChange({...config, mobs_timeout: value})} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Disconnect" helpText="Enable will automatically disconnect, otherwise we'll wait for revive" />}
                        item={<BooleanSlider value={config.on_death_disconnect ?? true} onChange={value => onChange?.({ ...config, on_death_disconnect: value })} />}
                    />
                </ConfigTable>
            }/>

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
                        item={<TimeInput value={config.afk_timeout} onChange={value => onChange({...config, afk_timeout: value})} />}
                    />

                </ConfigTable>
            }/>

            <Modal isShowing={statsModal.isShown} hide={statsModal.close}
            title={<h4>Stats</h4>} body={
                <div className="stats">
                    <div className="stats__state">
                        <span className={`stats__badge stats__badge--${botState}`}>{botState}</span>
                    </div>

                    <div className="stats__section">
                        <div className="stats__section-title">Session</div>
                        <div className="stats__grid">
                            <div className="stats__card">
                                <span className="stats__label">Kills</span>
                                <span className="stats__value">{info?.enemy_kill_count ?? 0}</span>
                            </div>
                            <div className="stats__card">
                                <span className="stats__label">Botting Time</span>
                                <span className="stats__value stats__value--time">{botStopWatch?.toString() ?? "00:00:00:000"}</span>
                            </div>
                        </div>
                    </div>

                    <div className="stats__section">
                        <div className="stats__section-title">Last Kill</div>
                        <div className="stats__grid">
                            <div className="stats__card">
                                <span className="stats__label">Search Time</span>
                                <span className="stats__value stats__value--time">{searchMobStopWatch}</span>
                            </div>
                            <div className="stats__card">
                                <span className="stats__label">Fight Time</span>
                                <span className="stats__value stats__value--time">{fightStopWatch}</span>
                            </div>
                        </div>
                    </div>

                    <div className="stats__section">
                        <div className="stats__section-title">Performance</div>
                        <div className="stats__grid">
                            <div className="stats__card">
                                <span className="stats__label">Last KPM</span>
                                <span className="stats__value">{info?.kill_min_avg?.toFixed(2) ?? 0}</span>
                            </div>
                            <div className="stats__card">
                                <span className="stats__label">Last KPH</span>
                                <span className="stats__value">{info?.kill_hour_avg?.toFixed(2) ?? 0}</span>
                            </div>
                            <div className="stats__card">
                                <span className="stats__label">Global KPM</span>
                                <span className="stats__value">{globalKPM === "NaN" || globalKPM === "Infinity" ? "0.00" : globalKPM}</span>
                            </div>
                            <div className="stats__card">
                                <span className="stats__label">Global KPH</span>
                                <span className="stats__value">{globalKPH === "NaN" || globalKPH === "Infinity" ? "0.00" : globalKPH}</span>
                            </div>
                        </div>
                    </div>
                </div>
            }/>
            {info && (
                <div className="info">
                    <div className="row">
                        <div>State: { botState }</div>
                    </div>
                    <div className="row">
                        <div>Target's detection mode: { config.is_stop_fighting? "🛑" : "✅" }</div>
                    </div>
                    <button className="btn sm"
                            onClick={e => onChange?.({ ...config, is_stop_fighting: !config.is_stop_fighting })} >
                            Detection 🎯
                    </button>
                    <div className="row">
                        <button className="btn sm" onClick={statsModal.open}>Stats 📊</button>
                        <button className="btn sm" onClick={debugModal.open}>Settings ⚙️</button>
                    </div>
                </div>
            )}
        </div>
    )
}

export default styled(FarmingConfig)`
    & .modalCenteredContent {
        text-align: center;
    }
`
