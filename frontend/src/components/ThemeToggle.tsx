import { useState } from 'react'
import type { EffectName } from './ParticleEffect'
import { PickerModal } from './PickerModal'

/** Maps each theme to its natural particle effect. */
export const THEME_EFFECT_MAP: Record<string, EffectName> = {
  dark: 'none',
  light: 'none',
  midnight: 'stars',
  forest: 'forest',
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
  br2049: 'wasteland',
  snow: 'snow',
  spring: 'sakura',
  summer: 'fireflies',
  autumn: 'leaves',
  christmas: 'snow',
  halloween: 'flames',
  newyear: 'fireworks',
  valentine: 'hearts',
  patriot: 'stars',
  boba: 'boba',
  fruitsalad: 'fruit',
  junkfood: 'junkfood',
  space: 'stars',
  neon: 'fireflies',
  candy: 'candy',
  retro: 'digital-rain',
  coffee: 'embers',
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
      { id: 'br2049', emoji: '🏜️', name: '2049' },
    ],
  },
  {
    label: 'Seasonal',
    themes: [
      { id: 'snow', emoji: '❄️', name: 'Winter' },
      { id: 'spring', emoji: '🌸', name: 'Spring' },
      { id: 'summer', emoji: '☀️', name: 'Summer' },
      { id: 'autumn', emoji: '🍁', name: 'Autumn' },
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
      { id: 'retro', emoji: '🕹️', name: 'Retro' },
      { id: 'coffee', emoji: '☕', name: 'Coffee' },
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

  function select(id: string) {
    onChange(id)
    localStorage.setItem(`theme:${username}`, id)
    document.documentElement.setAttribute('data-theme', id)

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

  function selectRandom() {
    const allThemes = THEME_GROUPS.flatMap(g => g.themes).filter(t => t.id !== current)
    const pick = allThemes[Math.floor(Math.random() * allThemes.length)]
    if (pick) select(pick.id)
  }

  const currentTheme = THEME_GROUPS.flatMap(g => g.themes).find(t => t.id === current)

  return (
    <>
      <PickerModal open={open} onClose={() => setOpen(false)} title="Choose Theme">
        <button className="picker-surprise-btn" onClick={selectRandom}>
          <span style={{ fontSize: '1rem' }}>🎲</span> Surprise Me
        </button>
        {THEME_GROUPS.map(group => (
          <div key={group.label} style={{ marginBottom: '0.75rem' }}>
            <div className="picker-group-label">{group.label}</div>
            <div className="picker-grid picker-grid-themes">
              {group.themes.map(t => (
                <button
                  key={t.id}
                  onClick={() => select(t.id)}
                  className={`picker-item ${t.id === current ? 'picker-item-active' : ''}`}
                  title={t.name}
                >
                  <span style={{ fontSize: '1.3rem' }}>{t.emoji}</span>
                  <span>{t.name}</span>
                </button>
              ))}
            </div>
          </div>
        ))}
      </PickerModal>

      <button
        onClick={() => setOpen(!open)}
        title={`Theme: ${currentTheme?.name ?? current}`}
        className="picker-fab"
        aria-label={`Theme: ${currentTheme?.name ?? current}. Click to open theme picker.`}
      >
        <i className="bi bi-palette" />
      </button>
    </>
  )
}
