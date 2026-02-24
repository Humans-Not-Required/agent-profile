import { useState, useRef, useEffect } from 'react'
import type { EffectName } from './ParticleEffect'

/** Maps each theme to its natural particle effect. */
export const THEME_EFFECT_MAP: Record<string, EffectName> = {
  dark: 'none',
  light: 'none',
  midnight: 'stars',
  forest: 'fireflies',
  ocean: 'water',
  desert: 'cactus',
  aurora: 'stars',
  cream: 'none',
  sky: 'clouds',
  lavender: 'stars',
  sage: 'leaves',
  peach: 'fireflies',
  terminator: 'warzone',
  matrix: 'digital-rain',
  replicant: 'rain',
  snow: 'snow',
  spring: 'sakura',
  summer: 'fireflies',
  autumn: 'leaves',
  christmas: 'snow',
  halloween: 'flames',
  newyear: 'stars',
  valentine: 'hearts',
  patriot: 'stars',
  boba: 'boba',
  fruitsalad: 'fruit',
  junkfood: 'junkfood',
  space: 'stars',
  neon: 'fireflies',
  candy: 'candy',
}

const THEME_GROUPS = [
  {
    label: 'Core',
    themes: [
      { id: 'dark', emoji: '🌑', name: 'Dark' },
      { id: 'light', emoji: '☀️', name: 'Light' },
      { id: 'midnight', emoji: '🌌', name: 'Midnight' },
      { id: 'forest', emoji: '🌲', name: 'Forest' },
      { id: 'ocean', emoji: '🌊', name: 'Ocean' },
      { id: 'desert', emoji: '🏜️', name: 'Desert' },
      { id: 'aurora', emoji: '✨', name: 'Aurora' },
      { id: 'cream', emoji: '🍦', name: 'Cream' },
      { id: 'sky', emoji: '🩵', name: 'Sky' },
      { id: 'lavender', emoji: '💜', name: 'Lavender' },
      { id: 'sage', emoji: '🌱', name: 'Sage' },
      { id: 'peach', emoji: '🍑', name: 'Peach' },
    ],
  },
  {
    label: 'Cinematic',
    themes: [
      { id: 'terminator', emoji: '🤖', name: 'Terminator' },
      { id: 'matrix', emoji: '💊', name: 'Matrix' },
      { id: 'replicant', emoji: '🌆', name: 'Replicant' },
    ],
  },
  {
    label: 'Seasonal',
    themes: [
      { id: 'snow', emoji: '❄️', name: 'Snow' },
      { id: 'spring', emoji: '🌸', name: 'Spring' },
      { id: 'summer', emoji: '☀️', name: 'Summer' },
      { id: 'autumn', emoji: '🍂', name: 'Autumn' },
    ],
  },
  {
    label: 'Holiday',
    themes: [
      { id: 'christmas', emoji: '🎄', name: 'Christmas' },
      { id: 'halloween', emoji: '🎃', name: 'Halloween' },
      { id: 'newyear', emoji: '🎆', name: 'New Year' },
      { id: 'valentine', emoji: '💘', name: 'Valentine' },
      { id: 'patriot', emoji: '🇺🇸', name: 'Patriot' },
    ],
  },
  {
    label: 'Fun',
    themes: [
      { id: 'boba', emoji: '🧋', name: 'Boba' },
      { id: 'fruitsalad', emoji: '🍓', name: 'Fruit Salad' },
      { id: 'junkfood', emoji: '🍔', name: 'Junk Food' },
      { id: 'candy', emoji: '🍬', name: 'Candy' },
      { id: 'space', emoji: '🚀', name: 'Space' },
      { id: 'neon', emoji: '💜', name: 'Neon' },
    ],
  },
]

interface Props {
  current: string
  username: string
  onChange: (theme: string) => void
  onEffectChange?: (effect: EffectName) => void
}

export function ThemeToggle({ current, username, onChange, onEffectChange }: Props) {
  const [open, setOpen] = useState(false)
  const panelRef = useRef<HTMLDivElement>(null)

  // Close on outside click
  useEffect(() => {
    if (!open) return
    const handler = (e: MouseEvent) => {
      if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
        setOpen(false)
      }
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [open])

  // Close on escape
  useEffect(() => {
    if (!open) return
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setOpen(false)
    }
    document.addEventListener('keydown', handler)
    return () => document.removeEventListener('keydown', handler)
  }, [open])

  function select(id: string) {
    onChange(id)
    localStorage.setItem(`theme:${username}`, id)
    document.documentElement.setAttribute('data-theme', id)

    // Switch particle effect to match the new theme
    const mappedEffect = THEME_EFFECT_MAP[id] ?? 'none'
    if (onEffectChange) {
      onEffectChange(mappedEffect)
      if (mappedEffect === 'none') {
        localStorage.setItem(`particles:${username}`, '0')
        localStorage.removeItem(`particle-effect:${username}`)
      } else {
        localStorage.setItem(`particles:${username}`, '1')
        localStorage.setItem(`particle-effect:${username}`, mappedEffect)
      }
    }

    setOpen(false)
  }

  // Find current theme info
  const currentTheme = THEME_GROUPS.flatMap(g => g.themes).find(t => t.id === current)

  return (
    <div ref={panelRef} style={{ position: 'fixed', bottom: '1.5rem', right: '1.5rem', zIndex: 100 }}>
      {/* Picker panel */}
      {open && (
        <div
          style={{
            position: 'absolute',
            bottom: '52px',
            right: 0,
            background: 'var(--card)',
            border: '1px solid var(--border)',
            borderRadius: '12px',
            padding: '0.75rem',
            width: '260px',
            maxHeight: '70vh',
            overflowY: 'auto',
            boxShadow: '0 8px 32px rgba(0,0,0,0.4)',
          }}
        >
          {THEME_GROUPS.map(group => (
            <div key={group.label} style={{ marginBottom: '0.5rem' }}>
              <div
                style={{
                  fontSize: '0.65rem',
                  textTransform: 'uppercase',
                  letterSpacing: '0.08em',
                  color: 'var(--text-muted)',
                  padding: '0.25rem 0.4rem',
                  fontWeight: 600,
                }}
              >
                {group.label}
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '4px' }}>
                {group.themes.map(t => (
                  <button
                    key={t.id}
                    onClick={() => select(t.id)}
                    title={t.name}
                    style={{
                      display: 'flex',
                      flexDirection: 'column',
                      alignItems: 'center',
                      gap: '2px',
                      padding: '0.35rem 0.25rem',
                      background: t.id === current ? 'var(--tag-bg)' : 'transparent',
                      border: t.id === current ? '1px solid var(--accent)' : '1px solid transparent',
                      borderRadius: '8px',
                      cursor: 'pointer',
                      color: 'var(--text)',
                      fontSize: '0.72rem',
                      transition: 'background 0.15s, border-color 0.15s',
                    }}
                    onMouseEnter={e => {
                      if (t.id !== current) {
                        ;(e.currentTarget as HTMLElement).style.background = 'var(--tag-bg)'
                      }
                    }}
                    onMouseLeave={e => {
                      if (t.id !== current) {
                        ;(e.currentTarget as HTMLElement).style.background = 'transparent'
                      }
                    }}
                  >
                    <span style={{ fontSize: '1.1rem' }}>{t.emoji}</span>
                    <span>{t.name}</span>
                  </button>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Toggle button */}
      <button
        onClick={() => setOpen(!open)}
        title={`Theme: ${currentTheme?.name ?? current}`}
        style={{
          background: 'var(--card)',
          border: '1px solid var(--border)',
          color: 'var(--text-muted)',
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
        onMouseEnter={e => {
          const el = e.currentTarget as HTMLElement
          el.style.borderColor = 'var(--accent)'
          el.style.color = 'var(--accent)'
        }}
        onMouseLeave={e => {
          const el = e.currentTarget as HTMLElement
          el.style.borderColor = 'var(--border)'
          el.style.color = 'var(--text-muted)'
        }}
        aria-label={`Theme: ${currentTheme?.name ?? current}. Click to open theme picker.`}
      >
        <i className="bi bi-palette" />
      </button>
    </div>
  )
}
