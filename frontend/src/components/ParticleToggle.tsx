interface Props {
  enabled: boolean
  effectName: string
  username: string
  onChange: (enabled: boolean) => void
}

export function ParticleToggle({ enabled, effectName, username, onChange }: Props) {
  if (effectName === 'none') return null

  function toggle() {
    const next = !enabled
    onChange(next)
    localStorage.setItem(`particles:${username}`, next ? '1' : '0')
  }

  return (
    <button
      onClick={toggle}
      title={enabled ? 'Disable particle effect' : 'Enable particle effect'}
      style={{
        position: 'fixed',
        bottom: '1.5rem',
        right: '4.5rem',
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
        zIndex: 100,
        boxShadow: '0 2px 8px rgba(0,0,0,0.3)',
      }}
      aria-label={enabled ? 'Disable particles' : 'Enable particles'}
    >
      <i className={`bi ${enabled ? 'bi-stars' : 'bi-circle'}`} />
    </button>
  )
}
