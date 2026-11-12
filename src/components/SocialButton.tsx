import { IconType } from 'react-icons/lib'
import styled from 'styled-components'

type Props = {
    className?: string,
    icon: IconType,
    label: string,
    href: string,
}

/** Small icon link; the label shows as a tooltip. */
const SocialButton = ({ className, icon: Icon, label, href }: Props) => (
    <a className={className} href={href} target="_blank" rel="noreferrer" title={label} aria-label={label}>
        <Icon size={13} />
    </a>
)

export default styled(SocialButton)`
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius);
    color: var(--text-faint);
    transition: background-color .12s ease, color .12s ease;

    &:hover {
        background: var(--surface-2);
        color: var(--text);
    }
`
