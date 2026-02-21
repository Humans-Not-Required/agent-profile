interface Props {
  avatarUrl: string
  displayName: string
  hue: number
  initials: string
}

export function Avatar({ avatarUrl, displayName, hue, initials }: Props) {
  const bgColor = `hsl(${hue}, 60%, 35%)`

  if (avatarUrl) {
    return (
      <img
        src={avatarUrl}
        alt={`${displayName}'s avatar`}
        className="avatar-img"
        onError={e => {
          // Fall back to initials placeholder on error
          const placeholder = document.createElement('div')
          placeholder.className = 'avatar-placeholder'
          placeholder.style.background = bgColor
          placeholder.textContent = initials
          e.currentTarget.replaceWith(placeholder)
        }}
      />
    )
  }

  return (
    <div
      className="avatar-placeholder"
      style={{ background: bgColor }}
    >
      {initials}
    </div>
  )
}
