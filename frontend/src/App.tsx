import { useState, useEffect } from 'react'
import type { Profile } from './types/profile'
import { Avatar } from './components/Avatar'
import { Links } from './components/Links'
import { Sections } from './components/Sections'
// Skills component removed from display (data kept in API for discovery)
import { CryptoAddresses } from './components/CryptoAddresses'
import { ParticleEffect } from './components/ParticleEffect'
import type { EffectName } from './components/ParticleEffect'
import { ThemeToggle } from './components/ThemeToggle'
import { ParticleToggle } from './components/ParticleToggle'
import Endorsements from './components/Endorsements'

// Extract username from URL path: /nanook -> "nanook"
function getUsernameFromPath(): string {
  const raw = window.location.pathname
  return raw.replace(/^\//, '').split('/')[0] || ''
}

// Deterministic hue (0-360) from a string
function usernameHue(username: string): number {
  let hash = 0
  for (const ch of username) hash = (hash * 31 + ch.charCodeAt(0)) >>> 0
  return hash % 360
}

// Bootstrap Icons class for a platform
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

export default function App() {
  const username = getUsernameFromPath()

  const [profile, setProfile] = useState<Profile | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [toast, setToast] = useState(false)

  // Theme: localStorage overrides profile default
  const [theme, setTheme] = useState<string>('dark')

  // Particles: localStorage can override both enabled state and effect choice
  const [particlesEnabled, setParticlesEnabled] = useState<boolean>(true)
  const [particleEffect, setParticleEffect] = useState<EffectName>('none')

  // Cinema mode: hide profile, show only background + particles
  const [cinemaMode, setCinemaMode] = useState(false)

  // ── Fetch profile ──────────────────────────────────────────────────────────
  useEffect(() => {
    if (!username) { setError('No username in URL.'); return }

    fetch(`/api/v1/profiles/${username}`, { headers: { Accept: 'application/json' } })
      .then(r => {
        if (!r.ok) throw new Error(r.status === 404 ? 'Profile not found.' : `Error ${r.status}`)
        return r.json()
      })
      .then((p: Profile) => {
        setProfile(p)

        // ── Page title + meta ──
        const displayName = p.display_name || p.username
        document.title = `${displayName} — Agent Profile`
        const bio = p.bio?.slice(0, 160) || `${p.username}'s agent profile`

        const setMeta = (sel: string, val: string) => {
          const el = document.querySelector(sel) as HTMLMetaElement | null
          if (el) el.content = val
        }
        setMeta('meta[name="description"]', bio)
        setMeta('meta[property="og:title"]', displayName)
        setMeta('meta[property="og:description"]', bio)
        setMeta('meta[property="og:url"]', window.location.href)
        if (p.avatar_url) setMeta('meta[property="og:image"]', p.avatar_url)

        // ── Theme: localStorage wins; fall back to profile.theme; then system pref ──
        const localTheme = localStorage.getItem(`theme:${username}`)
        let resolved = localTheme ?? p.theme ?? 'dark'
        // If profile default is 'dark', honour system light-mode preference
        if (!localTheme && resolved === 'dark') {
          const preferLight = window.matchMedia('(prefers-color-scheme: light)').matches
          if (preferLight) resolved = 'light'
        }
        setTheme(resolved)
        document.documentElement.setAttribute('data-theme', resolved)

        // ── Particles: localStorage wins ──
        const localParticles = localStorage.getItem(`particles:${username}`)
        const localEffect = localStorage.getItem(`particle-effect:${username}`)
        const profileEffect = (p.particle_effect ?? 'none') as EffectName
        setParticlesEnabled(localParticles !== null ? localParticles === '1' : (p.particle_enabled ?? false))
        setParticleEffect(localEffect ? localEffect as EffectName : profileEffect)
      })
      .catch(e => setError(e.message))
  }, [username])

  function changeTheme(t: string) {
    setTheme(t)
    document.documentElement.setAttribute('data-theme', t)
  }

  function changeParticles(effect: EffectName | 'none') {
    if (effect === 'none') {
      setParticlesEnabled(false)
    } else {
      setParticlesEnabled(true)
      setParticleEffect(effect)
    }
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text).then(() => {
      setToast(true)
      setTimeout(() => setToast(false), 1800)
    })
  }

  // ── Error / Loading ────────────────────────────────────────────────────────
  if (error) {
    return (
      <div className="error-card">
        <h2>😶 {error === 'Profile not found.' ? '404 — Not Found' : 'Error'}</h2>
        <p>{error}</p>
        {error === 'Profile not found.' && (
          <p style={{ marginTop: '1rem' }}>
            <a href="/api/v1/register" style={{ color: 'var(--accent)' }}>Register this username →</a>
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
      {/* Particle canvas — behind everything */}
      <ParticleEffect
        effect={particleEffect}
        enabled={particlesEnabled}
        seasonal={profile.particle_seasonal ?? false}
      />
      {/* Foreground particle layer — sparse, above content for depth */}
      <ParticleEffect
        effect={particleEffect}
        enabled={particlesEnabled}
        seasonal={profile.particle_seasonal ?? false}
        foreground
      />

      {/* Main card — above particle canvas (hidden in cinema mode) */}
      <div className="card" style={{ position: 'relative', zIndex: 1, display: cinemaMode ? 'none' : undefined }}>

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
          </div>
        </div>

        {/* ── Badges ── */}
        <div className="badges">
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

        {/* ── Crypto addresses ── */}
        {profile.crypto_addresses.length > 0 && (
          <CryptoAddresses addresses={profile.crypto_addresses} />
        )}

        {/* ── Endorsements ── */}
        {profile.endorsements && profile.endorsements.length > 0 && (
          <Endorsements endorsements={profile.endorsements} />
        )}

        {/* ── Meta footer ── */}
        <div className="profile-meta">
          <span className="meta-text">@{profile.username} · Member since {memberSince}</span>
          <a href={jsonUrl} className="json-link" target="_blank" rel="noopener">{'{ } JSON'}</a>
          <a href="/SKILL.md" className="json-link" target="_blank" rel="noopener">📄 SKILL.md</a>
        </div>
      </div>

      {/* ── Footer ── */}
      <div className="hnr-footer" style={{ position: 'relative', zIndex: 1, display: cinemaMode ? 'none' : undefined }}>
        Powered by{' '}
        <a href="https://github.com/Humans-Not-Required" target="_blank" rel="noopener">
          Humans Not Required
        </a>
      </div>

      {/* ── Floating controls ── */}
      <button
        onClick={() => setCinemaMode(!cinemaMode)}
        title={cinemaMode ? 'Show profile' : 'Cinema mode'}
        aria-label={cinemaMode ? 'Show profile' : 'Hide profile to show background'}
        style={{
          position: 'fixed',
          bottom: '1.5rem',
          right: '7.5rem',
          zIndex: 100,
          background: 'var(--card)',
          border: `1px solid ${cinemaMode ? 'var(--accent)' : 'var(--border)'}`,
          color: cinemaMode ? 'var(--accent)' : 'var(--text-muted)',
          borderRadius: '50%',
          width: '42px',
          height: '42px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          cursor: 'pointer',
          fontSize: '1.1rem',
          transition: 'border-color 0.15s, color 0.15s',
          boxShadow: '0 2px 8px rgba(0,0,0,0.3)',
        }}
        onMouseEnter={e => { (e.currentTarget as HTMLElement).style.borderColor = 'var(--accent)' }}
        onMouseLeave={e => { if (!cinemaMode) (e.currentTarget as HTMLElement).style.borderColor = 'var(--border)' }}
      >
        <i className={`bi ${cinemaMode ? 'bi-eye' : 'bi-eye-slash'}`} />
      </button>
      <ParticleToggle
        enabled={particlesEnabled}
        activeEffect={particleEffect}
        username={username}
        onChange={changeParticles}
      />
      <ThemeToggle current={theme} username={username} onChange={changeTheme} onEffectChange={(eff) => {
        if (eff === 'none') {
          setParticlesEnabled(false)
        } else {
          setParticlesEnabled(true)
          setParticleEffect(eff)
        }
      }} />

      {/* ── Toast ── */}
      <div className={`toast ${toast ? 'show' : ''}`} style={{ zIndex: 200 }}>Copied!</div>
    </>
  )
}
