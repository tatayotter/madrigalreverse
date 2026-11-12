import React, { ReactElement } from 'react'
import styled from 'styled-components'
import { LuX } from 'react-icons/lu'


type Props = {
    className?: string,
    isShowing: boolean,
    hide: () => void,
    body?: ReactElement;
    title?: ReactElement;
    closeBtn?: boolean;
    onHide?: () => void,

}

const SlotModal = ({className, isShowing, hide, body, title,closeBtn = true, onHide }: Props) => {
    const onHiding = () => {
        if(onHide) {
            onHide()
        }
        hide()
    }
    if (isShowing) {
        return(
            <div className={className} >
                <div className="modal-overlay">
                    <div className="modal-wrapper" onMouseDown={event => { if (event.target === event.currentTarget)  onHiding()}}>
                    <div className="modal">
                        <div className="modal-header">
                            {title &&
                                title
                            }
                            {closeBtn &&
                                <button
                                    type="button"
                                    className="modal-close-button"
                                    onClick={onHiding}
                                >
                                    <LuX size={16} />
                                </button>
                            }
                        </div>

                            {body &&
                               <div className="modal-body">{body}</div>
                            }

                    </div>
                    </div>
                </div>
            </div>
    )
    }else {
        return(<></>)
    }

}


export default styled(SlotModal)`
position: absolute;

& .modal-overlay {
    position: fixed;
    inset: 0;
    z-index: 1040;
    background-color: rgba(5, 6, 8, .6);
    backdrop-filter: blur(2px);
}

& .modal-wrapper {
    position: fixed;
    inset: 0;
    z-index: 1050;
    overflow-x: hidden;
    overflow-y: auto;
    outline: 0;
    display: flex;
    align-items: center;
    padding: 16px;
}

& .modal {
    position: relative;
    z-index: 100;
    margin: auto;
    width: 100%;
    max-width: 460px;
    max-height: calc(100vh - 32px);
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, .5);
    color: var(--text);
    overflow: hidden;
}

& .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding: 10px 10px 10px 16px;
    border-bottom: 1px solid var(--border);

    & h1, & h2, & h3, & h4 {
        font-size: 14px;
        font-weight: 600;
    }
}

& .modal-close-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    flex-shrink: 0;
    border: none;
    border-radius: var(--radius);
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;

    &:hover {
        background: var(--surface-2);
        color: var(--text);
    }
}

& .modal-body {
    padding: 12px 16px 16px;
    overflow-y: auto;
}

& .slot {
    text-align: center;
    background-color: var(--surface-2);
}

& .desc {
    width: 100%;
    text-align: center;
}

& img.type {
    width: 100%;
    height: 100%;
    object-fit: contain;
    padding: .25rem;
    border-radius: .25rem;
}
`
