interface Props {
  score: number
  color: string
}

export function ProfileScore({ score, color }: Props) {
  return (
    <span
      className="badge badge-score"
      style={{ '--score-color': color } as React.CSSProperties}
      title={`Profile completeness: ${score}/100`}
    >
      {score}% complete
    </span>
  )
}
