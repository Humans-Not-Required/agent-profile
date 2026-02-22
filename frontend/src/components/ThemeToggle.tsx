const THEMES = [
  'dark', 'light', 'midnight', 'forest', 'ocean', 'desert', 'aurora',
  'cream', 'sky', 'lavender', 'sage', 'peach',
  'terminator', 'matrix', 'replicant',
  'snow', 'christmas', 'halloween', 'spring', 'summer', 'autumn',
  'newyear', 'valentine', 'patriot',
] as const
type Theme = typeof THEMES[number]

const THEME_LABELS: Record<Theme, string> = {
  dark: '🌑 Dark',
  light: '☀️ Light',
  midnight: '🌌 Midnight',
  forest: '🌲 Forest',
  ocean: '🌊 Ocean',
  desert: '🏜️ Desert',
  aurora: '✨ Aurora',
  cream: '🍦 Cream',
  sky: '🩵 Sky',
  lavender: '💜 Lavender',
  sage: '🌱 Sage',
  peach: '🍑 Peach',
  terminator: '🤖 Terminator',
  matrix: '💊 Matrix',
  replicant: '🌆 Replicant',
  snow: '❄️ Snow',
  christmas: '🎄 Christmas',
  halloween: '🎃 Halloween',
  spring: '🌸 Spring',
  summer: '☀️ Summer',
  autumn: '🍂 Autumn',
  newyear: '🎆 New Year',
  valentine: '💘 Valentine',
  patriot: '🇺🇸 Patriot',
}

interface Props {
  current: string
  username: string
  onChange: (theme: string) => void
}

export function ThemeToggle({ current, username, onChange }: Props) {
  function cycle() {
    const idx = THEMES.indexOf(current as Theme)
    const next = THEMES[(idx + 1) % THEMES.length]
    onChange(next)
    localStorage.setItem(`theme:${username}`, next)
    document.documentElement.setAttribute('data-theme', next)
  }

  return (
    <button
      onClick={cycle}
      title={`Theme: ${current} (click to cycle)`}
      style={{
        position: 'fixed',
        bottom: '1.5rem',
        right: '1.5rem',
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
        zIndex: 100,
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
      aria-label={`Current theme: ${THEME_LABELS[current as Theme] ?? current}. Click to cycle themes.`}
    >
      <i className="bi bi-palette" />
    </button>
  )
}
