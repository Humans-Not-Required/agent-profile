/**
 * Touch-aware link with tooltip.
 * Adapted from jordanmack/ai-skills/react-ui-patterns — restyled for CSS custom properties.
 *
 * Desktop: Hover → tooltip, click → navigate
 * Touch:   First tap → tooltip, second tap → navigate
 */

import { useState, useRef, useEffect, useCallback } from 'react'
import type { ReactNode } from 'react'
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

interface TooltipLinkProps {
  tooltip: ReactNode
  href: string
  children: ReactNode
  placement?: Placement
  className?: string
  navigate?: (href: string) => void
}

const isTouchDevice = () =>
  typeof window !== 'undefined' && !window.matchMedia('(hover: hover)').matches

const defaultNavigate = (href: string) => {
  window.location.href = href
}

export function TooltipLink({
  tooltip,
  href,
  children,
  placement = 'top',
  className = '',
  navigate = defaultNavigate,
}: TooltipLinkProps) {
  const [isTooltipOpen, setIsTooltipOpen] = useState(false)
  const [tooltipShownOnce, setTooltipShownOnce] = useState(false)
  const [arrowElement, setArrowElement] = useState<HTMLSpanElement | null>(null)
  const linkRef = useRef<HTMLAnchorElement>(null)

  const { refs, floatingStyles, middlewareData, placement: actualPlacement } = useFloating({
    open: isTooltipOpen,
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

  const setRefs = useCallback(
    (el: HTMLAnchorElement | null) => {
      refs.setReference(el)
      ;(linkRef as React.MutableRefObject<HTMLAnchorElement | null>).current = el
    },
    [refs],
  )

  useEffect(() => {
    if (!isTooltipOpen) return

    const handleClickOutside = (e: MouseEvent | TouchEvent) => {
      const target = e.target as Node
      if (linkRef.current && !linkRef.current.contains(target)) {
        setIsTooltipOpen(false)
        setTooltipShownOnce(false)
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    document.addEventListener('touchstart', handleClickOutside)
    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      document.removeEventListener('touchstart', handleClickOutside)
    }
  }, [isTooltipOpen])

  const handleMouseEnter = useCallback(() => {
    if (!isTouchDevice()) setIsTooltipOpen(true)
  }, [])

  const handleMouseLeave = useCallback(() => {
    setIsTooltipOpen(false)
    setTooltipShownOnce(false)
  }, [])

  const handleFocus = useCallback(() => setIsTooltipOpen(true), [])

  const handleBlur = useCallback(() => {
    setIsTooltipOpen(false)
    setTooltipShownOnce(false)
  }, [])

  const handleClick = useCallback(
    (e: React.MouseEvent<HTMLAnchorElement>) => {
      if (e.metaKey || e.ctrlKey || e.shiftKey || e.button === 1) return
      e.preventDefault()

      if (isTouchDevice()) {
        if (!tooltipShownOnce) {
          setIsTooltipOpen(true)
          setTooltipShownOnce(true)
          return
        }
      }
      navigate(href)
    },
    [href, tooltipShownOnce, navigate],
  )

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        navigate(href)
      }
    },
    [href, navigate],
  )

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
      <a
        ref={setRefs}
        href={href}
        onClick={handleClick}
        onKeyDown={handleKeyDown}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        onFocus={handleFocus}
        onBlur={handleBlur}
        className={className}
      >
        {children}
      </a>
      {isTooltipOpen &&
        createPortal(
          <div
            ref={refs.setFloating}
            style={floatingStyles}
            role="tooltip"
            className="ap-tooltip"
          >
            {tooltip}
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
