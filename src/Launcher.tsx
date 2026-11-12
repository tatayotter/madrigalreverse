import { useReducer, useState, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api'
import styled from 'styled-components'

import MissionControl from './MissionControl'
import useModal from './components/utils/UseModal'
import { getVersion } from '@tauri-apps/api/app'
import YesNoModal from './components/YesNoModal'
import TextInput from './components/config/TextInput'
import { LuCopy, LuPencil, LuPlay, LuPlus, LuRefreshCw, LuRotateCcw, LuTrash2 } from 'react-icons/lu'

type Props = {
    className?: string,
}

const Launcher = ({ className }: Props) => {
    const [hasEnteredMainLoop, enterMainLoop] = useReducer(() => true, false);
    const [isLaunched, setIsLaunched] = useState(false)

    const [currentVersion, setCurrentVersion] = useState("NaN")
    const lastVersion = useRef("NaN")
    const updateModal = useModal()

    const newProfileModal = useModal()
    const renameProfileModal = useModal()
    const copyProfileModal = useModal()
    const resetProfileModal = useModal()


    const delProfileModal = useModal()

    const [profileId, setPID] = useState<string>("")
    const [idList, setList] = useState<string[]>(["DEFAULT"])
    const [newProfile,setNewProfile] = useState("")

    const hasProfile = profileId !== ""
    const sortedProfiles = [...idList].sort((a, b) => a.localeCompare(b))

    const launch = (pid: string = profileId) => {
        if (!hasEnteredMainLoop && pid !== "" ) {
            setPID(pid)
            enterMainLoop()
            invoke('create_window',{profileId: pid}).then(()=> {
                invoke('start_bot',{profileId: pid}).then(()=> {setIsLaunched(true)})
            })

        }
    }

    const getData=()=>{
        fetch('https://raw.githubusercontent.com/MadrigalStreetCartel/neuz/main/updater.json')
        .then(function(response){
            return response.json();
        })
        .then(function(myJson) {
            if(currentVersion !== "NaN" && myJson.version !== currentVersion) {
                lastVersion.current = myJson.version
                updateModal.open()
            }
        });
    }

    if (currentVersion === "NaN") {
        getVersion().then((value) => {
            setCurrentVersion(value)
        })

    }

    const refreshProfiles = () => {
        invoke('get_profiles').then((value: any)=> {
            if(value) setList(value);
        })
    }

    useEffect(() => {
        getData()
        refreshProfiles()

    },[currentVersion])

    return (
            <div className={className}>
                <YesNoModal isShowing={updateModal.isShown} hide={updateModal.close}
                    title={<h4>Update available</h4>}
                    body={
                        <>
                            <p>Version {lastVersion.current} is available (you have {currentVersion}).</p>
                            <p>Open the download page?</p>
                        </>
                    }
                    onYes={() => {window.open("https://github.com/MadrigalStreetCartel/neuz/releases/Neuz-v" + lastVersion.current)}}
                />

                <YesNoModal isShowing={newProfileModal.isShown} hide={newProfileModal.close}
                    title={<h4>New profile</h4>}
                    body={
                        <TextInput unit='#' value={newProfile} onChange={(value) => {setNewProfile(value.toUpperCase()) }} />
                    }
                    onYes={() => {
                        if (newProfile.length > 0 && !idList.includes("profile_" + newProfile.toUpperCase())){
                            invoke('create_profile', {profileId:newProfile.toUpperCase()})
                            setList((oldValue) => [...oldValue, "profile_" + newProfile.toUpperCase()])
                        }
                        setNewProfile("")
                    }}
                />

                <YesNoModal isShowing={renameProfileModal.isShown} hide={renameProfileModal.close}
                    title={<h4>Rename {profileId}</h4>}
                    body={
                        <TextInput unit='#' value={newProfile} onChange={(value) => {setNewProfile(value) }} />
                    }
                    onYes={() => {
                        if (!idList.includes("profile_" + newProfile.toUpperCase())){
                            invoke('rename_profile', {profileId: profileId, newProfileId: newProfile})
                            setList((oldValue) => [...oldValue.filter((val) => val.replaceAll("profile_", "") !== profileId), "profile_" + newProfile.toUpperCase()])
                        }
                        setNewProfile("")
                    }}
                />

                <YesNoModal isShowing={copyProfileModal.isShown} hide={copyProfileModal.close}
                    title={<h4>Copy {profileId} as</h4>}
                    body={
                        <TextInput unit='#' value={newProfile} onChange={(value) => {setNewProfile(value) }} />
                    }
                    onYes={() => {
                        if (!idList.includes("profile_" + newProfile.toUpperCase())){
                            invoke('copy_profile', {profileId: profileId, newProfileId: newProfile})
                            setList((oldValue) => [...oldValue, "profile_" + newProfile.toUpperCase()])
                        }
                        setNewProfile("")
                    }}
                />

                <YesNoModal isShowing={delProfileModal.isShown} hide={delProfileModal.close}
                    title={<h4>Delete {profileId}?</h4>}
                    body={
                       <p>This can't be undone.</p>
                    }
                    onYes={() => {
                        invoke('remove_profile', {profileId: profileId})
                        setList((oldValue) => oldValue.filter((filtred) => filtred.replaceAll("profile_","") !== profileId))
                        setPID("")
                    }}
                />

                <YesNoModal isShowing={resetProfileModal.isShown} hide={resetProfileModal.close}
                    title={<h4>Reset {profileId} to defaults?</h4>}
                    body={
                       <p>This can't be undone.</p>
                    }
                    onYes={() => {
                        invoke('reset_profile', {profileId: profileId})
                    }}
                />

                {!isLaunched && (
                    <div className="launcher">
                        <header className="launcher__header">
                            <div className="brand">
                                <span className="brand__name">Neuz</span>
                                <span className="brand__sub">Profile manager</span>
                            </div>
                            {currentVersion !== "NaN" && <span className="version">v{currentVersion}</span>}
                        </header>

                        <section className="panel">
                            <div className="panel__head">
                                <span className="panel__title">Profiles</span>
                                <button className="btn sm ghost" title="Reload the profile list" onClick={refreshProfiles}><LuRefreshCw size={13} /></button>
                            </div>

                            <ul className="profile-list">
                                {sortedProfiles.map((pid) => {
                                    const name = pid.replaceAll("profile_", "")
                                    return (
                                        <li key={pid}
                                            className={name === profileId ? "selected" : ""}
                                            onClick={() => setPID(name)}
                                            onDoubleClick={() => launch(name)}
                                            title="Double-click to start">
                                            {name}
                                        </li>
                                    )
                                })}
                            </ul>

                            <div className="profile-actions">
                                <button className="btn sm" onClick={newProfileModal.open}><LuPlus size={13} />New</button>
                                <button className={`btn sm ${hasProfile ? "" : "disabled"}`} onClick={() => hasProfile && renameProfileModal.open()}><LuPencil size={13} />Rename</button>
                                <button className={`btn sm ${hasProfile ? "" : "disabled"}`} onClick={() => hasProfile && copyProfileModal.open()}><LuCopy size={13} />Copy</button>
                                <button className={`btn sm ${hasProfile ? "" : "disabled"}`} onClick={() => hasProfile && resetProfileModal.open()}><LuRotateCcw size={13} />Reset</button>
                                <button className={`btn sm ${hasProfile ? "" : "disabled"}`} onClick={() => hasProfile && delProfileModal.open()}><LuTrash2 size={13} />Delete</button>
                            </div>
                        </section>

                        <button className={`btn lg primary block ${hasProfile ? "" : "disabled"}`} onClick={() => launch()}>
                            <LuPlay size={15} />{hasProfile ? `Start ${profileId}` : "Select a profile"}
                        </button>
                    </div>
                )}
                {isLaunched && (
                    <MissionControl profileId={profileId} currentVersion={currentVersion} lastVersion={lastVersion.current} />
                )}

            </div>
    )
}

export default styled(Launcher)`
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg);

    & .launcher {
        display: flex;
        flex-direction: column;
        gap: 12px;
        width: 100%;
        max-width: 420px;
        height: 100%;
        margin: 0 auto;
        padding: 20px 16px 16px;
    }

    & .launcher__header {
        display: flex;
        align-items: baseline;
        justify-content: space-between;
    }

    & .brand {
        display: flex;
        align-items: baseline;
        gap: 8px;
    }

    & .brand__name {
        font-size: 18px;
        font-weight: 700;
        letter-spacing: -.01em;
    }

    & .brand__sub,
    & .version {
        color: var(--text-muted);
        font-size: 12px;
    }

    & .panel {
        display: flex;
        flex-direction: column;
        flex: 1;
        min-height: 0;
        background: var(--surface);
        border: 1px solid var(--border);
        border-radius: var(--radius-lg);
        overflow: hidden;
    }

    & .panel__head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 6px 6px 6px 12px;
        border-bottom: 1px solid var(--border);
    }

    & .panel__title {
        font-size: 11px;
        font-weight: 600;
        letter-spacing: .06em;
        text-transform: uppercase;
        color: var(--text-muted);
    }

    & .profile-list {
        list-style: none;
        margin: 0;
        padding: 4px;
        flex: 1;
        overflow-y: auto;

        & li {
            display: flex;
            align-items: center;
            height: 32px;
            padding: 0 10px;
            border-radius: var(--radius);
            font-weight: 500;
            cursor: pointer;
            border: 1px solid transparent;

            &:hover {
                background: var(--surface-2);
            }

            &.selected {
                background: var(--accent-soft);
                border-color: rgba(76, 141, 255, .35);
                color: #fff;
            }
        }
    }

    & .profile-actions {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
        padding: 8px;
        border-top: 1px solid var(--border);

        & .btn {
            flex: 1;
        }
    }
`
