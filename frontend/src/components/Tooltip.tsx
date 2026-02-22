/**
 * Accessible tooltip with smart positioning using Floating UI.
 * Adapted from jordanmack/ai-skills/react-ui-patterns — restyled for CSS custom properties.
 *
 * Features:
 * - Flips to opposite side when near viewport edge
 * - Shifts horizontally to stay in view
 * - Arrow points to trigger element
 * - Portal rendering to avoid z-index issues
 * - Touch-friendly interactive mode
 */

import { useState, useRef, useEffect, cloneElement, useCallback } from 'react'
import type { ReactNode, ReactElement } from 'react'
import { createPortal } from 'react-dom'
import {
  useFloating,
  autoUpdate,
  offset,
  flip,
  shift,
  arrow,
  size,
} from '@floating-ui/react-dom'
import type { Placement } from '@floating-ui/react-dom'

interface TooltipProps {
  content: ReactNode
  children: ReactElement<any>
  placement?: Placement
  disabled?: boolean
  interactive?: boolean
}

const isTouchDevice = () =>
  typeof window !== 'undefined' && !window.matchMedia('(hover: hover)').matches

export function Tooltip({
  content,
  children,
  placement = 'top',
  disabled = false,
  interactive = false,
}: TooltipProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [arrowElement, setArrowElement] = useState<HTMLSpanElement | null>(null)
  const referenceRef = useRef<HTMLElement>(null)

  const { refs, floatingStyles, middlewareData, placement: actualPlacement } = useFloating({
    open: isOpen,
    placement,
    whileElementsMounted: autoUpdate,
    middleware: [
      offset(8),
      flip({ fallbackAxisSideDirection: 'start' }),
      shift({ padding: 8 }),
      size({
        apply({ availableWidth, elements }) {
          Object.assign(elements.floating.style, {
            maxWidth: `${Math.max(200, availableWidth - 16)}px`,
          })
        },
        padding: 8,
      }),
      arrow({ element: arrowElement }),
    ],
  })

  useEffect(() => {
    if (!interactive || !isOpen) return

    const handleClickOutside = (e: MouseEvent | TouchEvent) => {
      const target = e.target as Node
      if (referenceRef.current && !referenceRef.current.contains(target)) {
        setIsOpen(false)
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    document.addEventListener('touchstart', handleClickOutside)
    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      document.removeEventListener('touchstart', handleClickOutside)
    }
  }, [interactive, isOpen])

  const setRefs = useCallback(
    (el: HTMLElement | null) => {
      refs.setReference(el)
      ;(referenceRef as React.MutableRefObject<HTMLElement | null>).current = el
    },
    [refs],
  )

  if (disabled || !content) return children

  const handleClick = (e: React.MouseEvent) => {
    if (interactive && isTouchDevice() && !isOpen) {
      e.preventDefault()
      setIsOpen(true)
      return
    }
    children.props.onClick?.(e)
  }

  const arrowX = middlewareData.arrow?.x
  const arrowY = middlewareData.arrow?.y
  const side = actualPlacement.split('-')[0] as 'top' | 'bottom' | 'left' | 'right'

  const arrowSide = {
    top: 'bottom',
    right: 'left',
    bottom: 'top',
    left: 'right',
  }[side] as string

  return (
    <>
      {cloneElement(children, {
        ref: setRefs,
        onMouseEnter: (e: React.MouseEvent) => {
          if (!(interactive && isTouchDevice())) setIsOpen(true)
          children.props.onMouseEnter?.(e)
        },
        onMouseLeave: (e: React.MouseEvent) => {
          setIsOpen(false)
          children.props.onMouseLeave?.(e)
        },
        onFocus: (e: React.FocusEvent) => {
          setIsOpen(true)
          children.props.onFocus?.(e)
        },
        onBlur: (e: React.FocusEvent) => {
          setIsOpen(false)
          children.props.onBlur?.(e)
        },
        onClick: handleClick,
      })}
      {isOpen &&
        createPortal(
          <div
            ref={refs.setFloating}
            style={floatingStyles}
            role="tooltip"
            className="ap-tooltip"
          >
            {content}
            <span
              ref={setArrowElement}
              className="ap-tooltip-arrow"
              style={{
                position: 'absolute',
                left: arrowX != null ? `${arrowX}px` : '',
                top: arrowY != null ? `${arrowY}px` : '',
                [arrowSide]: '-4px',
              }}
            />
          </div>,
          document.body,
        )}
    </>
  )
}
