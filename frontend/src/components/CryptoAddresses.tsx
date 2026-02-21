import type { CryptoAddress } from '../types/profile'

interface Props {
  addresses: CryptoAddress[]
  onCopy: (text: string) => void
}

function networkEmoji(network: string): string {
  const map: Record<string, string> = {
    bitcoin: '₿',
    lightning: '⚡',
    ethereum: 'Ξ',
    cardano: '₳',
    ergo: 'ERG',
    nervos: 'CKB',
    solana: '◎',
    monero: 'ɱ',
    dogecoin: 'Ð',
    nostr: '🔑',
    custom: '🔗',
  }
  return map[network] ?? '🔗'
}

export function CryptoAddresses({ addresses, onCopy }: Props) {
  return (
    <div className="section">
      <h3 className="section-title">Crypto Addresses</h3>
      <div className="addr-list">
        {addresses.map(a => {
          const short = a.address.length > 20
            ? `${a.address.slice(0, 10)}…${a.address.slice(-8)}`
            : a.address
          return (
            <div
              key={a.id}
              className="addr-row"
              onClick={() => onCopy(a.address)}
              title={`Click to copy: ${a.address}`}
            >
              <span style={{ fontSize: '1rem', flexShrink: 0 }}>{networkEmoji(a.network)}</span>
              <span className="addr-network">{a.network}</span>
              {a.label && <span className="addr-label">({a.label})</span>}
              <code className="addr-value">{short}</code>
              <span className="addr-copy-hint">click to copy</span>
            </div>
          )
        })}
      </div>
    </div>
  )
}
