import styled from 'styled-components'
import Modal from './Modal'
import { ReactElement } from 'react'

type Props = {
    className?: string,
    isShowing: boolean,
    title?: ReactElement,
    body?: ReactElement,
    hide: () => void,
    onYes: () => void,
    onNo?: () => void,
}

const YesNoModal = ({className, isShowing, hide, onYes, onNo, title, body}: Props) => {
    const onClick = (kind: number) => {
        if(kind === 0) {
            onYes()
        } else if (kind == 1 && onNo) {
            onNo()
        }
        hide()
    }
    return(
        <div className={className}>
            <Modal isShowing={isShowing} onHide={() => onClick(1)} hide={hide} title={title ?? <h4>Confirm</h4>} body={
                <>
                    {body && <div className='modalBody'> {body} </div>}
                    <div className="buttonHolder">
                        <button className='btn' onClick={() => onClick(1)}>Cancel</button>
                        <button className='btn primary' onClick={() => onClick(0)}>Confirm</button>
                    </div>
                </>
            }/>
        </div>

    )
}


export default styled(YesNoModal)`
    & .modalBody {
        margin-bottom: 14px;
        color: var(--text-muted);
    }

    & .buttonHolder {
        display: flex;
        justify-content: flex-end;
        gap: 8px;
    }
`
