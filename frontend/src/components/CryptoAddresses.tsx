import type { CryptoAddress } from '../types/profile'
import { AddressDisplay } from './AddressDisplay'

interface Props {
  addresses: CryptoAddress[]
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

export function CryptoAddresses({ addresses }: Props) {
  return (
    <div className="section">
      <h3 className="section-title">Crypto Addresses</h3>
      <div className="addr-list">
        {addresses.map(a => (
          <div key={a.id} className="addr-row">
            <span className="addr-emoji">{networkEmoji(a.network)}</span>
            <span className="addr-network">{a.network}</span>
            {a.label && <span className="addr-label">({a.label})</span>}
            <AddressDisplay
              address={a.address}
              truncate
              prefixLen={10}
              suffixLen={6}
              forceMobileTruncation
              className="addr-value-wrap"
            />
          </div>
        ))}
      </div>
    </div>
  )
}
