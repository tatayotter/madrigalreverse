import styled from "styled-components"
import { listen, emit } from '@tauri-apps/api/event'
import { useEffect, useState } from "react"
import { isNil } from 'lodash'

import TabControl from "./components/TabControl"
import Tab from "./components/Tab"


import { BotConfigModel, ModeModel } from './models/BotConfig'
import FarmingConfig from "./components/behaviors/FarmingConfig"
import SupportConfig from "./components/behaviors/SupportConfig"
import ShoutConfig from "./components/behaviors/ShoutConfig"
import Footer from "./components/Footer"
import { FrontendInfoModel } from "./models/FrontendInfo"
import { invoke } from "@tauri-apps/api"
import { writeText } from "@tauri-apps/api/clipboard"
import { useStopWatch } from "./components/utils/StopWatch"
import { LuAppWindow, LuCheck, LuCopy, LuCrosshair, LuFolderOpen, LuHeartPulse, LuMaximize2, LuMegaphone, LuMinimize2, LuPlay, LuSquare } from "react-icons/lu"

//type Bounds = {x: number, y: number, w: number, h: number}

type Props = {
    className?: string,
    profileId?: string,
    currentVersion?: string,
    lastVersion?: string,
}

const MissionControl = ({ className, profileId = "", lastVersion, currentVersion }: Props) => {
    const [logsCopied, setLogsCopied] = useState(false)
    const copyLogs = () => {
        invoke<string>("get_logs", { lines: 300 })
            .then(text => writeText(text))
            .then(() => {
                setLogsCopied(true)
                setTimeout(() => setLogsCopied(false), 2000)
            })
            .catch(e => console.error("Could not copy logs", e))
    }
    //const [imageData, setImageData] = useState({ data: '', width: 0, height: 0 })
    const [info, setInfo] = useState<FrontendInfoModel | null>(null);
    const [config, setConfig] = useState<BotConfigModel | null>(null);

    useEffect(() => {
        listen<string>('bot_visualizer_update', event => {
        /*  const payload = event.payload as unknown as string[]
            const data = payload[0]
            const width = Number(payload[1])
            const height = Number(payload[2])
            setImageData({ data, width, height }) */
        })

        listen('bot_info_s2c', event => {
            const payload = event.payload as FrontendInfoModel
            setInfo(payload)
        })

        listen('bot_config_s2c', event => {
            const payload = event.payload as BotConfigModel
            setConfig(payload)
        })
    }, [])

    const setRunningToggle = () => {
        if (!config) return
        const newConfig = { ...config, is_running: !config.is_running }
        emit('bot_config_c2s', newConfig)
    }

    const handleToggle = () => {
        if (!config) return
        const newConfig = { ...config }
        newConfig.farming_config.is_stop_fighting = !newConfig.farming_config.is_stop_fighting
        emit('bot_config_c2s', newConfig)
    }


    const handleTabSelect = (mode: ModeModel) => {
        const newConfig = { ...config, mode }
        emit('bot_config_c2s', newConfig)
    }

    const makeConfigUpdater = (key: string) => <T,>(patchedConfig: T) => {
        const newConfig = { ...config, [key]: patchedConfig }
        emit('bot_config_c2s', newConfig)
    }

    const [lightMode, setLightMode] = useState(false)
    const minSizeFarm = [250, 200]
    const minSize = [250, 160]
    const toogleLightMode = () => invoke("toggle_main_size", {size: config?.mode === "Farming"? minSizeFarm : minSize}).then((value) => setLightMode(value as boolean))

    const farmingState = config?.is_running? info?.is_running? !info?.is_alive? "dead" : info.is_attacking? "fighting" : "searching" : "ready" : "idle"
    const farmStopWatch = useStopWatch(farmingState === "searching" || farmingState === "fighting")

    const supportState = config?.is_running? info?.is_running? !info?.is_alive? "dead" : "healing" : "ready" : "idle"
    const suppStopWatch = useStopWatch(supportState === "healing")
    const activeState = config?.mode === "Farming" ? farmingState : config?.mode === "Support" ? supportState : null
    const engageBtn = (extra: string = "") => (
        <button className={`btn ${config?.is_running ? "danger" : "success"} ${extra}`} onClick={setRunningToggle}>
            {config?.is_running ? <><LuSquare size={13} />Stop</> : <><LuPlay size={14} />Start</>}
        </button>
    )

    return (
        <div className={className}>
            {!lightMode && <div className="vstack">
                <header className="topbar">
                    <div className="topbar__profile">
                        <span className="topbar__label">Profile</span>
                        <span className="topbar__name">{profileId}</span>
                    </div>
                    {activeState && <span className={`badge badge--${activeState}`}>{activeState}</span>}
                </header>

                {config && (
                    <>
                        <TabControl activeMode={config.mode} onSelect={handleTabSelect}>
                            <Tab mode="Farming" label="Farming" icon={LuCrosshair} />
                            <Tab mode="Support" label="Support" icon={LuHeartPulse} />
                            <Tab mode="AutoShout" label="Shout" icon={LuMegaphone} />
                        </TabControl>
                        <div className="config-container">
                            {config?.mode === 'Farming' && (<FarmingConfig botState={farmingState} botStopWatch={farmStopWatch.watch} info={info} config={config.farming_config} onChange={makeConfigUpdater('farming_config')} />)}
                            {config?.mode === 'Support' && (<SupportConfig botState={supportState} botStopWatch={suppStopWatch.watch} info={info} config={config.support_config} onChange={makeConfigUpdater('support_config')} />)}
                            {config?.mode === 'AutoShout' && (<ShoutConfig config={config.shout_config} onChange={makeConfigUpdater('shout_config')} />)}
                        </div>
                    </>
                )}

                <div className="footer">
                    <div className="toolbar">
                        {!isNil(config?.mode) && <button className="btn sm ghost" title="Shrink this window to a small control strip" onClick={toogleLightMode}><LuMinimize2 size={13} />Compact</button>}
                        {!isNil(config?.mode) && <button className="btn sm ghost" title="Bring the game window to the front" onClick={() => {invoke("focus_client")}}><LuAppWindow size={13} />Focus</button>}
                        <button className="btn sm ghost" title="Copy the latest log lines to paste into a bug report" onClick={copyLogs}>{logsCopied ? <><LuCheck size={13} />Copied</> : <><LuCopy size={13} />Copy logs</>}</button>
                        <button className="btn sm ghost" title="Open the folder with the log files" onClick={() => {invoke("open_log_folder")}}><LuFolderOpen size={13} />Logs</button>
                    </div>
                    {!isNil(config?.mode) && engageBtn("lg block")}
                </div>
                <Footer currentVersion={currentVersion} lastVersion={lastVersion} />
            </div>}

            {lightMode &&
                <div className="lightmode">
                    <div className="lightmode__head">
                        <button className="btn sm ghost icon" title="Back to the full window" onClick={toogleLightMode}><LuMaximize2 size={13} /></button>
                        <span className="lightmode__name">{profileId}</span>
                        {activeState && <span className={`badge badge--${activeState}`}>{activeState}</span>}
                    </div>
                    <div className="lightmode__row">
                        <button className="btn sm" onClick={() => {invoke("focus_client")}}><LuAppWindow size={13} />Focus</button>
                        {config?.mode === "Farming" && (
                            <button className={`btn sm ${config?.farming_config.is_stop_fighting ? "" : "on"}`}
                                title="Turn target detection on or off"
                                onClick={handleToggle}>
                                <LuCrosshair size={13} />{config?.farming_config.is_stop_fighting ? "Targets off" : "Targets on"}
                            </button>
                        )}
                    </div>
                    {engageBtn("block")}
                </div>
            }
        </div>
    )
}

export default styled(MissionControl)`
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg);

    & .vstack {
        display: flex;
        flex-direction: column;
        gap: 10px;
        height: 100%;
        padding: 10px 12px 0;
    }

    & .topbar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 8px;
        min-height: 24px;
    }

    & .topbar__profile {
        display: flex;
        align-items: baseline;
        gap: 8px;
        min-width: 0;
    }

    & .topbar__label {
        font-size: 11px;
        font-weight: 600;
        letter-spacing: .06em;
        text-transform: uppercase;
        color: var(--text-faint);
    }

    & .topbar__name {
        font-size: 14px;
        font-weight: 600;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    & .config-container {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        align-items: stretch;
        gap: 10px;
    }

    /* Status card under the slot bar (Farming / Support) */
    & .info {
        display: flex;
        flex-direction: column;
        gap: 2px;
        padding: 4px 12px;
        background: var(--surface);
        border: 1px solid var(--border);
        border-radius: var(--radius-lg);

        & .row {
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 8px;
            min-height: 36px;
            border-bottom: 1px solid var(--border);

            &:last-child {
                border-bottom: none;
            }
        }

        & .row__label {
            color: var(--text-muted);
        }

        & .row__value {
            font-family: var(--mono);
            font-size: 12px;
        }

        & .row--actions {
            justify-content: stretch;
            padding: 6px 0;

            & .btn {
                flex: 1;
            }
        }
    }

    /* Stats modal */
    & .stats {
        display: flex;
        flex-direction: column;
        gap: 12px;

        & .stats__state {
            display: flex;
            justify-content: flex-start;
        }

        & .stats__badge {
            display: inline-flex;
            align-items: center;
            gap: .35rem;
            height: 20px;
            padding: 0 8px;
            border-radius: 999px;
            font-size: 11px;
            font-weight: 600;
            letter-spacing: .03em;
            text-transform: uppercase;
            color: var(--text-muted);
            background: var(--surface-3);

            &::before {
                content: "";
                width: 6px;
                height: 6px;
                border-radius: 50%;
                background: currentColor;
            }

            &--searching { color: var(--warning); background: var(--warning-soft); }
            &--fighting { color: var(--danger); background: var(--danger-soft); }
            &--dead { color: var(--text-faint); background: var(--surface-2); }
            &--ready { color: var(--accent); background: var(--accent-soft); }
        }

        & .stats__section {
            display: flex;
            flex-direction: column;
            gap: 6px;
        }

        & .stats__section-title {
            font-size: 11px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: .06em;
            color: var(--text-faint);
        }

        & .stats__grid {
            display: grid;
            grid-template-columns: repeat(2, minmax(0, 1fr));
            gap: 6px;
        }

        & .stats__card {
            display: flex;
            flex-direction: column;
            gap: 2px;
            padding: 8px 10px;
            background: var(--surface-2);
            border: 1px solid var(--border);
            border-radius: var(--radius);
        }

        & .stats__label {
            font-size: 11px;
            color: var(--text-muted);
        }

        & .stats__value {
            font-size: 16px;
            font-weight: 600;
            font-variant-numeric: tabular-nums;

            &--time {
                font-size: 13px;
                font-family: var(--mono);
            }
        }
    }

    & .container {
        overflow: hidden;
        position: relative;
    }

    & .view {
        margin: 0;
        padding: 0;
        object-fit: fill;
    }

    & .enemy-tag {
        display: block;
        position: absolute;
        width: 4px;
        height: 4px;
        margin-top: -2px;
        margin-left: -2px;
        border-radius: 10rem;
        background-color: #ff0000;
        z-index: 1000;
    }

    & .enemy-rect {
        display: block;
        position: absolute;
        border: 2px solid #00ff00;
        z-index: 2000;
    }

    & .enemy-target {
        display: block;
        position: absolute;
        width: 10px;
        height: 10px;
        margin-top: -5px;
        margin-left: -5px;
        border-radius: 10rem;
        background-color: white;
        z-index: 3000;
    }

    & .footer {
        display: flex;
        flex-direction: column;
        gap: 8px;
        padding-top: 10px;
        border-top: 1px solid var(--border);
    }

    & .toolbar {
        display: flex;
        gap: 4px;

        & .btn {
            flex: 1;
            padding: 0 6px;
        }
    }

    /* Compact strip (window shrunk to about 250x200) */
    & .lightmode {
        display: flex;
        flex-direction: column;
        gap: 8px;
        height: 100%;
        padding: 8px;
    }

    & .lightmode__head {
        display: flex;
        align-items: center;
        gap: 6px;
        min-width: 0;
    }

    & .lightmode__name {
        flex: 1;
        min-width: 0;
        font-weight: 600;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    & .lightmode__row {
        display: flex;
        gap: 6px;

        & .btn {
            flex: 1;
            padding: 0 6px;
        }

        & .btn.on {
            color: var(--success);
            border-color: rgba(47, 191, 113, .4);
        }
    }

    & .btn.icon {
        width: 28px;
        padding: 0;
    }
`
