import type { ProfileSection } from '../types/profile'
import ReactMarkdown from 'react-markdown'
import rehypeSanitize from 'rehype-sanitize'

interface Props {
  sections: ProfileSection[]
}

export function Sections({ sections }: Props) {
  return (
    <>
      {sections.map(s => (
        <div key={s.id} className="section">
          <h3 className="section-title">{s.title}</h3>
          <div className="section-content">
            <ReactMarkdown
              rehypePlugins={[rehypeSanitize]}
              components={{
                // Open all links in new tab
                a: ({ children, href, ...props }) => (
                  <a href={href} target="_blank" rel="noopener noreferrer" className="section-link" {...props}>
                    {children}
                  </a>
                ),
                // Prevent nested headers from conflicting with section-title
                h1: ({ children }) => <strong className="md-heading">{children}</strong>,
                h2: ({ children }) => <strong className="md-heading">{children}</strong>,
                h3: ({ children }) => <strong className="md-heading">{children}</strong>,
              }}
            >
              {s.content}
            </ReactMarkdown>
          </div>
        </div>
      ))}
    </>
  )
}
