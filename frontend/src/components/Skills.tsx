import type { ProfileSkill } from '../types/profile'

interface Props {
  skills: ProfileSkill[]
}

export function Skills({ skills }: Props) {
  return (
    <div className="section">
      <h3 className="section-title">Skills</h3>
      <div className="skill-tags">
        {skills.map(s => (
          <span key={s.id} className="skill-tag">{s.skill}</span>
        ))}
      </div>
    </div>
  )
}
