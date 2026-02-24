import type { ProfileLink } from '../types/profile'

interface Props {
  links: ProfileLink[]
  platformIcon: (platform: string) => string
}

/**
 * Determine the best icon for a link by checking URL patterns first,
 * then falling back to the platform field.
 */
export function smartIcon(link: ProfileLink, platformIcon: (p: string) => string): string {
  const url = link.url.toLowerCase()
  const label = link.label.toLowerCase()

  // URL-based detection (takes priority over platform field)
  if (url.startsWith('mailto:')) return 'bi-envelope-at'
  if (url.includes('github.com')) return 'bi-github'
  if (url.includes('moltbook.com')) return 'bi-bug'  // 🦞 closest critter in Bootstrap Icons
  if (url.includes('twitter.com') || url.includes('x.com')) return 'bi-twitter-x'
  if (url.includes('t.me') || url.includes('telegram')) return 'bi-telegram'
  if (url.includes('discord.gg') || url.includes('discord.com')) return 'bi-discord'
  if (url.includes('youtube.com') || url.includes('youtu.be')) return 'bi-youtube'
  if (url.includes('linkedin.com')) return 'bi-linkedin'
  if (url.includes('mastodon') || url.includes('fosstodon')) return 'bi-mastodon'
  if (url.startsWith('nostr:') || url.includes('njump.me')) return 'bi-broadcast'

  // Label-based fallback
  if (label.includes('email') || label.includes('mail')) return 'bi-envelope-at'
  if (label.includes('moltbook') || label.includes('molt')) return 'bi-bug'
  if (label.includes('nostr')) return 'bi-broadcast'
  if (label.includes('blog')) return 'bi-journal-text'
  if (label.includes('docs') || label.includes('documentation')) return 'bi-book'

  // Platform field fallback
  return platformIcon(link.platform)
}

/**
 * Build a descriptive display string for a link.
 * Instead of just "GitHub", show "GitHub: nanookclaw"
 * Instead of just "Email", show "Email: nanook@claw.inc"
 */
function smartLabel(link: ProfileLink): { primary: string; detail: string | null } {
  const url = link.url
  const label = link.label

  // Email links — show the address
  if (url.startsWith('mailto:')) {
    const addr = url.replace('mailto:', '').split('?')[0]
    return { primary: 'Email', detail: addr }
  }

  // GitHub links — extract user/org from URL
  if (url.includes('github.com')) {
    const match = url.match(/github\.com\/([^/?#]+(?:\/[^/?#]+)?)/)
    if (match) {
      const path = match[1]
      // If label already contains the path info, don't duplicate
      if (label.toLowerCase().includes(path.toLowerCase().split('/')[0])) {
        return { primary: label, detail: null }
      }
      return { primary: label || 'GitHub', detail: path }
    }
  }

  // Moltbook — extract agent name
  if (url.includes('moltbook.com')) {
    const match = url.match(/moltbook\.com\/agent\/([^/?#]+)/)
    if (match) {
      return { primary: 'Moltbook', detail: `@${match[1]}` }
    }
  }

  // Telegram — extract bot/user handle
  if (url.includes('t.me/')) {
    const match = url.match(/t\.me\/([^/?#]+)/)
    if (match) {
      return { primary: 'Telegram', detail: `@${match[1]}` }
    }
  }

  // Nostr — show truncated npub/hex
  if (url.startsWith('nostr:')) {
    const id = url.replace('nostr:', '')
    const truncated = id.length > 16 ? `${id.slice(0, 8)}…${id.slice(-8)}` : id
    return { primary: 'Nostr', detail: truncated }
  }

  // Discord invite
  if (url.includes('discord.gg/') || url.includes('discord.com/invite/')) {
    const match = url.match(/(?:discord\.gg|discord\.com\/invite)\/([^/?#]+)/)
    if (match) {
      return { primary: 'Discord', detail: match[1] }
    }
  }

  // Generic website — show domain
  if (url.startsWith('http')) {
    try {
      const domain = new URL(url).hostname.replace('www.', '')
      // Only add domain if label doesn't already hint at it
      if (!label.toLowerCase().includes(domain.split('.')[0])) {
        return { primary: label, detail: domain }
      }
    } catch {
      // invalid URL, just use label
    }
  }

  return { primary: label, detail: null }
}

export function Links({ links, platformIcon }: Props) {
  return (
    <div className="section">
      <h3 className="section-title">Links</h3>
      <div className="links-row">
        {links.map(l => {
          const icon = smartIcon(l, platformIcon)
          const { primary, detail } = smartLabel(l)
          return (
            <a
              key={l.id}
              href={l.url}
              className="profile-link"
              target="_blank"
              rel="noopener noreferrer"
            >
              <i className={`bi ${icon} platform-icon`} />
              <span className="link-label">
                {primary}
                {detail && <span className="link-detail">: {detail}</span>}
              </span>
            </a>
          )
        })}
      </div>
    </div>
  )
}
