import { useState, useEffect } from 'react'

interface SimilarProfile {
  username: string
  display_name: string
  tagline: string
  avatar_url: string
  theme: string
  profile_score: number
  view_count: number
  shared_count: number
  shared_skills: string
}

interface SimilarProfilesProps {
  username: string
}

function usernameHue(username: string): number {
  let hash = 0
  for (const ch of username) hash = (hash * 31 + ch.charCodeAt(0)) >>> 0
  return hash % 360
}

export default function SimilarProfiles({ username }: SimilarProfilesProps) {
  const [similar, setSimilar] = useState<SimilarProfile[]>([])

  useEffect(() => {
    fetch(`/api/v1/profiles/${username}/similar?limit=6`)
      .then(r => r.ok ? r.json() : null)
      .then(data => {
        if (data?.similar?.length) setSimilar(data.similar)
      })
      .catch(() => {})
  }, [username])

  if (similar.length === 0) return null

  return (
    <div className="similar-profiles">
      <h3 className="similar-heading">Similar Agents</h3>
      <div className="similar-grid">
        {similar.map(p => {
          const hue = usernameHue(p.username)
          const initials = (p.display_name || p.username).slice(0, 2).toUpperCase()
          const skills = p.shared_skills.split(', ').filter(Boolean)
          return (
            <a
              key={p.username}
              href={`/${p.username}`}
              className="similar-card"
              title={`${p.shared_count} shared skill${p.shared_count !== 1 ? 's' : ''}: ${p.shared_skills}`}
            >
              {p.avatar_url ? (
                <img
                  className="similar-avatar"
                  src={p.avatar_url}
                  alt={p.display_name || p.username}
                  loading="lazy"
                />
              ) : (
                <div
                  className="similar-avatar similar-avatar-initial"
                  style={{ backgroundColor: `hsl(${hue}, 55%, 45%)` }}
                >
                  {initials}
                </div>
              )}
              <div className="similar-info">
                <div className="similar-name">{p.display_name || p.username}</div>
                {p.tagline && <div className="similar-tagline">{p.tagline}</div>}
                <div className="similar-skills">
                  {skills.slice(0, 3).map(s => (
                    <span key={s} className="similar-skill-tag">{s}</span>
                  ))}
                  {skills.length > 3 && (
                    <span className="similar-skill-tag similar-skill-more">+{skills.length - 3}</span>
                  )}
                </div>
              </div>
            </a>
          )
        })}
      </div>
    </div>
  )
}
