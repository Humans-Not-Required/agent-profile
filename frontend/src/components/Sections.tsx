import type { ProfileSection } from '../types/profile'

interface Props {
  sections: ProfileSection[]
}

export function Sections({ sections }: Props) {
  return (
    <>
      {sections.map(s => (
        <div key={s.id} className="section">
          <h3 className="section-title">{s.title}</h3>
          <div className="section-content">{s.content}</div>
        </div>
      ))}
    </>
  )
}
