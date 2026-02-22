/**
 * Address display with truncation, tooltip, copy button.
 * Adapted from jordanmack/ai-skills/react-ui-patterns — restyled for CSS custom properties.
 *
 * - Mobile: JS truncation (prefix…suffix)
 * - Desktop with truncate=true: CSS ellipsis on overflow
 * - Hover shows full address in tooltip
 * - Copy button with clipboard fallback
 */

import { useState, useCallback } from 'react'
import { Tooltip } from './Tooltip'
import { truncateAddress as truncateAddr } from './format'

interface AddressDisplayProps {
  address: string
  linkTo?: string
  truncate?: boolean
  prefixLen?: number
  suffixLen?: number
  forceMobileTruncation?: boolean
  className?: string
}

function useIsMobile(): boolean {
  if (typeof window === 'undefined') return false
  return window.innerWidth < 640
}

async function copyToClipboard(text: string): Promise<void> {
  if (navigator.clipboard) {
    await navigator.clipboard.writeText(text)
    return
  }
  const textarea = document.createElement('textarea')
  textarea.value = text
  document.body.appendChild(textarea)
  textarea.select()
  document.execCommand('copy')
  document.body.removeChild(textarea)
}

export function AddressDisplay({
  address,
  linkTo,
  truncate = true,
  prefixLen = 10,
  suffixLen = 6,
  forceMobileTruncation = false,
  className = '',
}: AddressDisplayProps) {
  const [copied, setCopied] = useState(false)
  const isMobile = useIsMobile()

  const shouldJsTruncate = isMobile || forceMobileTruncation
  const displayAddress = shouldJsTruncate ? truncateAddr(address, prefixLen, suffixLen) : address

  const handleCopy = useCallback(async () => {
    await copyToClipboard(address)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }, [address])

  const handleCopyKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        handleCopy()
      }
    },
    [handleCopy],
  )

  const cssTruncateClass = !isMobile && !forceMobileTruncation && truncate
    ? 'addr-display-ellipsis'
    : ''

  return (
    <span className={`addr-display ${className}`}>
      {linkTo ? (
        <Tooltip content={address}>
          <a
            href={linkTo}
            className={`addr-display-link ${cssTruncateClass}`}
            target="_blank"
            rel="noopener noreferrer"
          >
            {displayAddress}
          </a>
        </Tooltip>
      ) : (
        <Tooltip content={address}>
          <span className={`addr-display-text ${cssTruncateClass}`}>{displayAddress}</span>
        </Tooltip>
      )}

      <Tooltip content={copied ? 'Copied!' : 'Copy'} interactive>
        <span
          role="button"
          tabIndex={0}
          onClick={handleCopy}
          onKeyDown={handleCopyKeyDown}
          className="addr-copy-btn"
        >
          {copied ? (
            <svg className="addr-copy-icon addr-copy-ok" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
          ) : (
            <svg className="addr-copy-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          )}
        </span>
      </Tooltip>
    </span>
  )
}
