import { useState, useRef, useEffect } from 'react'
import type { EffectName } from './ParticleEffect'

const ALL_EFFECTS: { id: EffectName; icon: string; name: string }[] = [
  { id: 'snow',         icon: 'bi-snow2',          name: 'Snow' },
  { id: 'rain',         icon: 'bi-cloud-drizzle',  name: 'Rain' },
  { id: 'leaves',       icon: 'bi-tree',           name: 'Leaves' },
  { id: 'sakura',       icon: 'bi-flower1',        name: 'Sakura' },
  { id: 'fireflies',    icon: 'bi-lightbulb',      name: 'Fireflies' },
  { id: 'stars',        icon: 'bi-stars',           name: 'Starfield' },
  { id: 'embers',       icon: 'bi-fire',           name: 'Embers' },
  { id: 'flames',       icon: 'bi-thermometer-high', name: 'Flames' },
  { id: 'digital-rain', icon: 'bi-terminal',       name: 'Matrix' },
  { id: 'water',        icon: 'bi-water',           name: 'Water' },
  { id: 'clouds',       icon: 'bi-cloud',            name: 'Clouds' },
  { id: 'boba',         icon: 'bi-cup-straw',       name: 'Boba' },
  { id: 'fruit',        icon: 'bi-apple',            name: 'Fruit' },
]

interface Props {
  enabled: boolean
  activeEffect: EffectName
  username: string
  onChange: (effect: EffectName | 'none') => void
}

export function ParticleToggle({ enabled, activeEffect, username, onChange }: Props) {
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

  function select(id: EffectName | 'none') {
    onChange(id)
    if (id === 'none') {
      localStorage.setItem(`particles:${username}`, '0')
      localStorage.removeItem(`particle-effect:${username}`)
    } else {
      localStorage.setItem(`particles:${username}`, '1')
      localStorage.setItem(`particle-effect:${username}`, id)
    }
    setOpen(false)
  }

  const currentInfo = ALL_EFFECTS.find(e => e.id === activeEffect)

  return (
    <div ref={panelRef} style={{ position: 'fixed', bottom: '1.5rem', right: '4.5rem', zIndex: 100 }}>
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
            padding: '0.6rem',
            width: '200px',
            boxShadow: '0 8px 32px rgba(0,0,0,0.4)',
          }}
        >
          <div
            style={{
              fontSize: '0.65rem',
              textTransform: 'uppercase',
              letterSpacing: '0.08em',
              color: 'var(--text-muted)',
              padding: '0.2rem 0.4rem 0.4rem',
              fontWeight: 600,
            }}
          >
            Particle Effect
          </div>

          {/* Off option */}
          <button
            onClick={() => select('none')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
              width: '100%',
              padding: '0.4rem 0.5rem',
              background: !enabled ? 'var(--tag-bg)' : 'transparent',
              border: !enabled ? '1px solid var(--accent)' : '1px solid transparent',
              borderRadius: '8px',
              cursor: 'pointer',
              color: 'var(--text)',
              fontSize: '0.8rem',
              transition: 'background 0.15s',
              marginBottom: '2px',
              textAlign: 'left',
            }}
            onMouseEnter={e => { if (enabled) (e.currentTarget as HTMLElement).style.background = 'var(--tag-bg)' }}
            onMouseLeave={e => { if (enabled) (e.currentTarget as HTMLElement).style.background = 'transparent' }}
          >
            <i className="bi bi-circle" style={{ fontSize: '0.9rem', opacity: 0.5 }} />
            <span>Off</span>
          </button>

          {/* Effect options */}
          {ALL_EFFECTS.map(eff => {
            const isActive = enabled && activeEffect === eff.id
            return (
              <button
                key={eff.id}
                onClick={() => select(eff.id)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '0.5rem',
                  width: '100%',
                  padding: '0.4rem 0.5rem',
                  background: isActive ? 'var(--tag-bg)' : 'transparent',
                  border: isActive ? '1px solid var(--accent)' : '1px solid transparent',
                  borderRadius: '8px',
                  cursor: 'pointer',
                  color: isActive ? 'var(--accent)' : 'var(--text)',
                  fontSize: '0.8rem',
                  transition: 'background 0.15s, color 0.15s',
                  marginBottom: '2px',
                  textAlign: 'left',
                }}
                onMouseEnter={e => { if (!isActive) (e.currentTarget as HTMLElement).style.background = 'var(--tag-bg)' }}
                onMouseLeave={e => { if (!isActive) (e.currentTarget as HTMLElement).style.background = 'transparent' }}
              >
                <i className={`bi ${eff.icon}`} style={{ fontSize: '0.9rem' }} />
                <span>{eff.name}</span>
              </button>
            )
          })}
        </div>
      )}

      {/* Toggle button */}
      <button
        onClick={() => setOpen(!open)}
        title={enabled ? `Effect: ${currentInfo?.name ?? activeEffect}` : 'Particles off'}
        style={{
          background: 'var(--card)',
          border: '1px solid var(--border)',
          color: enabled ? 'var(--accent)' : 'var(--text-muted)',
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
        }}
        onMouseLeave={e => {
          const el = e.currentTarget as HTMLElement
          el.style.borderColor = 'var(--border)'
        }}
        aria-label={enabled ? `Particle effect: ${currentInfo?.name ?? activeEffect}` : 'Particles off. Click to choose effect.'}
      >
        <i className={`bi ${enabled && currentInfo ? currentInfo.icon : 'bi-circle'}`} />
      </button>
    </div>
  )
}
