import type { ProfileSection } from '../types/profile'

interface Props {
  sections: ProfileSection[]
}

/** Minimal content formatter: newlines → <br>, URLs → links, **bold**, *italic* */
function formatContent(text: string): (string | JSX.Element)[] {
  if (!text) return []

  const parts: (string | JSX.Element)[] = []
  // Split into lines for paragraph handling
  const lines = text.split('\n')
  let key = 0

  for (let i = 0; i < lines.length; i++) {
    if (i > 0) parts.push(<br key={`br-${key++}`} />)

    // Process each line: find URLs and wrap them in <a> tags
    const urlRegex = /(https?:\/\/[^\s<>"{}|\\^`[\]]+)/g
    const line = lines[i]
    let lastIndex = 0
    let match: RegExpExecArray | null

    while ((match = urlRegex.exec(line)) !== null) {
      // Text before the URL
      if (match.index > lastIndex) {
        parts.push(...formatInline(line.slice(lastIndex, match.index), key))
        key += 10
      }
      // The URL itself
      const url = match[1]
      parts.push(
        <a key={`url-${key++}`} href={url} target="_blank" rel="noopener noreferrer" className="section-link">
          {url.length > 60 ? url.slice(0, 57) + '…' : url}
        </a>
      )
      lastIndex = match.index + match[0].length
    }

    // Remaining text after last URL
    if (lastIndex < line.length) {
      parts.push(...formatInline(line.slice(lastIndex), key))
      key += 10
    }
  }

  return parts
}

/** Handle **bold** and *italic* in a text fragment */
function formatInline(text: string, baseKey: number): (string | JSX.Element)[] {
  const parts: (string | JSX.Element)[] = []
  // Match **bold** and *italic* (simple, non-nested)
  const regex = /(\*\*(.+?)\*\*|\*(.+?)\*)/g
  let lastIndex = 0
  let match: RegExpExecArray | null
  let k = baseKey

  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parts.push(text.slice(lastIndex, match.index))
    }
    if (match[2]) {
      // **bold**
      parts.push(<strong key={`b-${k++}`}>{match[2]}</strong>)
    } else if (match[3]) {
      // *italic*
      parts.push(<em key={`i-${k++}`}>{match[3]}</em>)
    }
    lastIndex = match.index + match[0].length
  }

  if (lastIndex < text.length) {
    parts.push(text.slice(lastIndex))
  }

  return parts
}

export function Sections({ sections }: Props) {
  return (
    <>
      {sections.map(s => (
        <div key={s.id} className="section">
          <h3 className="section-title">{s.title}</h3>
          <div className="section-content">{formatContent(s.content)}</div>
        </div>
      ))}
    </>
  )
}
