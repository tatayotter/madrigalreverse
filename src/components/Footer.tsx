import styled from 'styled-components'
import { BsDiscord, BsGithub } from 'react-icons/bs'

import SocialButton from './SocialButton'

type Props = {
    className?: string,
    lastVersion?: string,
    currentVersion?: string,
}

const Footer = ({ className, lastVersion: newVersion = "NaN", currentVersion = "NaN" }: Props) => {
    const hasUpdate = newVersion !== "NaN" && currentVersion !== "NaN" && newVersion !== currentVersion
    return (
        <footer className={className}>
            {currentVersion !== "NaN" && <span className="version">v{currentVersion}</span>}
            {hasUpdate && (
                <a className="update" target="_blank" rel="noreferrer"
                    href={"https://github.com/MadrigalStreetCartel/neuz/releases/tag/Neuz-v" + newVersion}>
                    Update {newVersion}
                </a>
            )}
            <div className="links">
                <SocialButton icon={BsDiscord} label="Discord" href="https://discord.gg/cZr3X3mCnq" />
                <SocialButton icon={BsGithub} label="GitHub" href="https://github.com/MadrigalStreetCartel/neuz" />
            </div>
        </footer>
    )
}

export default styled(Footer)`
    display: flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    margin: 0 -12px;
    padding: 0 12px;
    border-top: 1px solid var(--border);
    background: var(--surface);
    font-size: 11px;
    color: var(--text-faint);

    & .update {
        padding: 1px 6px;
        border-radius: 999px;
        background: var(--warning-soft);
        color: var(--warning);
        text-decoration: none;
        font-weight: 600;
    }

    & .links {
        display: flex;
        gap: 2px;
        margin-left: auto;
    }
`
