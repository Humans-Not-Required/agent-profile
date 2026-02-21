import { Endorsement } from '../types/profile';

interface EndorsementsProps {
  endorsements: Endorsement[];
}

function timeAgo(isoDate: string): string {
  const now = Date.now();
  const then = new Date(isoDate).getTime();
  const diffMs = now - then;
  const diffDays = Math.floor(diffMs / 86400000);
  if (diffDays === 0) return 'today';
  if (diffDays === 1) return 'yesterday';
  if (diffDays < 30) return `${diffDays}d ago`;
  const diffMonths = Math.floor(diffDays / 30);
  if (diffMonths < 12) return `${diffMonths}mo ago`;
  return `${Math.floor(diffMonths / 12)}y ago`;
}

export default function Endorsements({ endorsements }: EndorsementsProps) {
  if (!endorsements || endorsements.length === 0) return null;

  const verifiedCount = endorsements.filter(e => e.verified).length;

  return (
    <section className="profile-section">
      <h2 className="section-title">
        <i className="bi bi-patch-check-fill me-2" style={{ color: 'var(--accent)' }}></i>
        Endorsements
        <span className="ms-2 badge-count">{endorsements.length}</span>
        {verifiedCount > 0 && (
          <span
            className="ms-2"
            style={{ fontSize: '0.75rem', color: 'var(--accent)', opacity: 0.8 }}
            title={`${verifiedCount} cryptographically verified`}
          >
            {verifiedCount} verified
          </span>
        )}
      </h2>

      <div className="endorsements-list">
        {endorsements.map(endorsement => (
          <div key={endorsement.id} className="endorsement-card">
            {/* Avatar placeholder — initials from endorser username */}
            <div
              className="endorser-avatar"
              style={{ backgroundColor: usernameColor(endorsement.endorser_username) }}
              aria-hidden="true"
            >
              {endorsement.endorser_username.slice(0, 2).toUpperCase()}
            </div>

            <div className="endorsement-body">
              <div className="endorsement-header">
                <a
                  href={`/${endorsement.endorser_username}`}
                  className="endorser-username"
                  title={`View ${endorsement.endorser_username}'s profile`}
                >
                  @{endorsement.endorser_username}
                </a>
                {endorsement.verified && (
                  <span
                    className="verified-badge"
                    title="Cryptographically verified — signed with endorser's secp256k1 key"
                  >
                    <i className="bi bi-patch-check-fill"></i>
                  </span>
                )}
                <span className="endorsement-time">{timeAgo(endorsement.created_at)}</span>
              </div>
              <p className="endorsement-message">{endorsement.message}</p>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}

/** Deterministic color from username string */
function usernameColor(username: string): string {
  let hash = 0;
  for (let i = 0; i < username.length; i++) {
    hash = username.charCodeAt(i) + ((hash << 5) - hash);
  }
  const hue = Math.abs(hash) % 360;
  return `hsl(${hue}, 55%, 38%)`;
}
