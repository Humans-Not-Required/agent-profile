import type { ProfileLink } from '../types/profile'

interface Props {
  links: ProfileLink[]
  platformIcon: (platform: string) => string
}

export function Links({ links, platformIcon }: Props) {
  return (
    <div className="section">
      <h3 className="section-title">Links</h3>
      <div className="links-row">
        {links.map(l => (
          <a
            key={l.id}
            href={l.url}
            className="profile-link"
            target="_blank"
            rel="noopener noreferrer"
          >
            <i className={`bi ${platformIcon(l.platform)} platform-icon`} />
            {l.label}
          </a>
        ))}
      </div>
    </div>
  )
}
