/**
 * Formatting utilities for address/hash display.
 * Adapted from jordanmack/ai-skills/react-ui-patterns
 */

/**
 * Truncate a hex string for display (e.g., "0x12345678...12345678").
 */
export function truncateHex(hex: string, prefixLen = 8, suffixLen = 8): string {
  if (hex.length <= prefixLen + suffixLen + 4) return hex
  const prefix = hex.startsWith('0x') ? hex.slice(0, 2 + prefixLen) : hex.slice(0, prefixLen)
  const suffix = hex.slice(-suffixLen)
  return `${prefix}…${suffix}`
}

/**
 * Truncate an address for display.
 */
export function truncateAddress(address: string, prefixLen = 10, suffixLen = 6): string {
  if (address.length <= prefixLen + suffixLen + 3) return address
  return `${address.slice(0, prefixLen)}…${address.slice(-suffixLen)}`
}
