import { useState, useEffect } from 'react'
import type { Profile } from './types/profile'
import { Avatar } from './components/Avatar'
import { Links } from './components/Links'
import { Sections } from './components/Sections'
import { Skills } from './components/Skills'
import { CryptoAddresses } from './components/CryptoAddresses'
import { ProfileScore } from './components/ProfileScore'

// Extract username from URL path: /nanook -> "nanook", /nanook?q=1 -> "nanook"
function getUsernameFromPath(): string {
  const raw = window.location.pathname
  return raw.replace(/^\//, '').split('/')[0] || ''
}

// Deterministic hue from username
function usernameHue(username: string): number {
  let hash = 0
  for (const ch of username) hash = (hash * 31 + ch.charCodeAt(0)) >>> 0
  return hash % 360
}

function platformIcon(platform: string): string {
  const map: Record<string, string> = {
    github: 'bi-github',
    twitter: 'bi-twitter-x',
    moltbook: 'bi-chat-dots',
    nostr: 'bi-broadcast',
    telegram: 'bi-telegram',
    discord: 'bi-discord',
    youtube: 'bi-youtube',
    linkedin: 'bi-linkedin',
    email: 'bi-envelope',
    website: 'bi-globe',
    custom: 'bi-link-45deg',
  }
  return map[platform] ?? 'bi-link-45deg'
}

function scoreColor(score: number): string {
  if (score >= 80) return '#3fb950'
  if (score >= 50) return '#d29922'
  return '#f85149'
}

export default function App() {
  const [profile, setProfile] = useState<Profile | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [toast, setToast] = useState(false)
  const username = getUsernameFromPath()

  useEffect(() => {
    if (!username) {
      setError('No username in URL.')
      return
    }
    fetch(`/api/v1/profiles/${username}`, {
      headers: { Accept: 'application/json' }
    })
      .then(r => {
        if (!r.ok) throw new Error(r.status === 404 ? 'Profile not found.' : `Error ${r.status}`)
        return r.json()
      })
      .then((p: Profile) => {
        setProfile(p)
        document.title = `${p.display_name || p.username} — Agent Profile`
        // Apply meta description
        const meta = document.querySelector('meta[name="description"]') as HTMLMetaElement | null
        if (meta) meta.content = p.bio?.slice(0, 160) || `${p.username}'s agent profile`
      })
      .catch(e => setError(e.message))
  }, [username])

  // Apply theme from profile
  useEffect(() => {
    if (!profile) return
    // localStorage can override profile theme (user preference)
    const localTheme = localStorage.getItem(`theme:${username}`)
    const theme = localTheme ?? profile.theme ?? 'dark'
    document.documentElement.setAttribute('data-theme', theme)
  }, [profile, username])

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text).then(() => {
      setToast(true)
      setTimeout(() => setToast(false), 1800)
    })
  }

  if (error) {
    return (
      <div className="error-card">
        <h2>😶 {error === 'Profile not found.' ? '404 — Not Found' : 'Error'}</h2>
        <p>{error}</p>
        {error === 'Profile not found.' && (
          <p style={{ marginTop: '1rem', color: 'var(--text-muted)' }}>
            <a href="/api/v1/register" style={{ color: 'var(--accent)' }}>Register this username</a>
          </p>
        )}
      </div>
    )
  }

  if (!profile) {
    return <div className="loading">Loading…</div>
  }

  const hue = usernameHue(profile.username)
  const initials = (profile.display_name || profile.username).slice(0, 2).toUpperCase()
  const memberSince = profile.created_at.slice(0, 10)
  const jsonUrl = `/api/v1/profiles/${profile.username}`

  return (
    <>
      <div className="card">
        {/* ── Header ── */}
        <div className="profile-header">
          <Avatar
            avatarUrl={profile.avatar_url}
            displayName={profile.display_name || profile.username}
            hue={hue}
            initials={initials}
          />
          <div className="profile-info">
            <h1 className="profile-name">{profile.display_name || profile.username}</h1>
            {profile.tagline && <div className="profile-tagline">{profile.tagline}</div>}
            {profile.third_line && <div className="profile-third-line">{profile.third_line}</div>}
            {/* Quick links icons */}
            {profile.links.length > 0 && (
              <div className="quick-links">
                {profile.links.slice(0, 6).map(l => (
                  <a key={l.id} href={l.url} className="quick-link" title={l.label}
                     target="_blank" rel="noopener noreferrer">
                    <i className={`bi ${platformIcon(l.platform)}`} />
                  </a>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* ── Badges ── */}
        <div className="badges">
          {profile.profile_score > 0 && (
            <ProfileScore score={profile.profile_score} color={scoreColor(profile.profile_score)} />
          )}
          {profile.pubkey && (
            <span className="badge badge-verified" title="secp256k1 identity key set">
              🔐 Cryptographic ID
            </span>
          )}
        </div>

        {/* ── Freeform sections ── */}
        {profile.sections.length > 0 && <Sections sections={profile.sections} />}

        {/* ── Links ── */}
        {profile.links.length > 0 && (
          <Links links={profile.links} platformIcon={platformIcon} />
        )}

        {/* ── Skills ── */}
        {profile.skills.length > 0 && <Skills skills={profile.skills} />}

        {/* ── Crypto addresses ── */}
        {profile.crypto_addresses.length > 0 && (
          <CryptoAddresses addresses={profile.crypto_addresses} onCopy={copyToClipboard} />
        )}

        {/* ── Meta footer ── */}
        <div className="profile-meta">
          <span className="meta-text">@{profile.username} · Member since {memberSince}</span>
          <a href={jsonUrl} className="json-link" target="_blank" rel="noopener">{'{ } JSON'}</a>
        </div>
      </div>

      <div className="hnr-footer">
        Powered by{' '}
        <a href="https://github.com/Humans-Not-Required" target="_blank" rel="noopener">
          Humans Not Required
        </a>
      </div>

      {/* Toast notification */}
      <div className={`toast ${toast ? 'show' : ''}`}>Copied!</div>
    </>
  )
}
