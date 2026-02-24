import { useState } from 'react'
import type { EffectName } from './ParticleEffect'
import { PickerModal } from './PickerModal'

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
  { id: 'warzone',      icon: 'bi-crosshair',        name: 'Warzone' },
  { id: 'boba',         icon: 'bi-cup-straw',       name: 'Boba' },
  { id: 'fruit',        icon: 'bi-apple',            name: 'Fruit' },
  { id: 'junkfood',    icon: 'bi-basket2',          name: 'Junk Food' },
  { id: 'hearts',      icon: 'bi-heart-fill',       name: 'Hearts' },
  { id: 'cactus',      icon: 'bi-sun',              name: 'Cactus' },
  { id: 'candy',       icon: 'bi-gift',             name: 'Candy' },
]

interface Props {
  enabled: boolean
  activeEffect: EffectName
  username: string
  onChange: (effect: EffectName | 'none') => void
}

export function ParticleToggle({ enabled, activeEffect, username, onChange }: Props) {
  const [open, setOpen] = useState(false)

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
    <>
      <PickerModal open={open} onClose={() => setOpen(false)} title="Particle Effect">
        {/* Off option */}
        <button
          onClick={() => select('none')}
          className={`picker-effect-item ${!enabled ? 'picker-item-active' : ''}`}
        >
          <i className="bi bi-circle" style={{ fontSize: '1rem', opacity: 0.5 }} />
          <span>Off</span>
        </button>

        {/* Effect options */}
        <div className="picker-grid picker-grid-effects">
          {ALL_EFFECTS.map(eff => {
            const isActive = enabled && activeEffect === eff.id
            return (
              <button
                key={eff.id}
                onClick={() => select(eff.id)}
                className={`picker-effect-item ${isActive ? 'picker-item-active' : ''}`}
              >
                <i className={`bi ${eff.icon}`} style={{ fontSize: '1rem' }} />
                <span>{eff.name}</span>
              </button>
            )
          })}
        </div>
      </PickerModal>

      <button
        onClick={() => setOpen(!open)}
        title={enabled ? `Effect: ${currentInfo?.name ?? activeEffect}` : 'Particles off'}
        className="picker-fab picker-fab-secondary"
        aria-label={enabled ? `Particle effect: ${currentInfo?.name ?? activeEffect}` : 'Particles off. Click to choose effect.'}
      >
        <i className={`bi ${enabled && currentInfo ? currentInfo.icon : 'bi-circle'}`} />
      </button>
    </>
  )
}
