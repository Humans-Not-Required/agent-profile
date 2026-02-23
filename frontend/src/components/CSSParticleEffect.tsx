import { useMemo } from 'react'

// ── CSS-based emoji particle effects ──────────────────────────────────────────
// GPU-composited via CSS transform animations — zero JS per frame.
// Used for: leaves, snow, fruit, junkfood, sakura

interface Props {
  effect: 'leaves' | 'snow' | 'fruit' | 'junkfood' | 'sakura'
  foreground?: boolean
}

interface EmojiParticle {
  id: number
  emoji: string
  x: number        // start % from left
  size: number     // font-size in px
  duration: number  // animation duration in seconds
  delay: number     // animation-delay in seconds
  sway: number      // horizontal sway amplitude in vw
  startY: number    // start offset (negative = above viewport)
  opacity: number
  rotation: number  // start rotation deg
  rotationEnd: number // end rotation deg
}

const LEAF_EMOJI = ['🍁', '🍂', '🍃']
const SNOW_EMOJI = ['❄', '❅', '❆']
const FRUIT_EMOJI = ['🍎', '🍊', '🍋', '🍇', '🍓', '🍑', '🍌', '🍉', '🥝', '🍒', '🫐', '🍍']
const JUNKFOOD_EMOJI = [
  '🍕', '🍔', '🌭', '🍟', '🌮', '🌯', '🍗', '🍖', '🥓',
  '🧀', '🍩', '🍪', '🎂', '🍰', '🧁', '🍫', '🍬', '🍭',
  '🥤', '🍦', '🥞', '🧇', '🥨', '🍿', '🥡', '🥪',
]
const SAKURA_EMOJI = ['🌸']

function getConfig(effect: Props['effect'], foreground: boolean) {
  if (foreground) {
    // Foreground: very few, large, mostly invisible (matches 85% invisible pattern)
    return {
      count: effect === 'snow' ? 4 : 2,
      sizeMin: 40, sizeMax: 80,
      durationMin: 12, durationMax: 25,
      opacityMin: 0, opacityMax: 0.9,
      invisibleChance: 0.85,
      swayMin: 2, swayMax: 8,
    }
  }
  switch (effect) {
    case 'leaves':  return { count: 30, sizeMin: 18, sizeMax: 36, durationMin: 8, durationMax: 18, opacityMin: 0.4, opacityMax: 0.9, invisibleChance: 0, swayMin: 3, swayMax: 12 }
    case 'snow':    return { count: 35, sizeMin: 10, sizeMax: 28, durationMin: 10, durationMax: 25, opacityMin: 0.04, opacityMax: 0.12, invisibleChance: 0, swayMin: 2, swayMax: 6 }
    case 'fruit':   return { count: 50, sizeMin: 28, sizeMax: 56, durationMin: 6, durationMax: 14, opacityMin: 0.5, opacityMax: 1.0, invisibleChance: 0, swayMin: 3, swayMax: 10 }
    case 'junkfood': return { count: 55, sizeMin: 24, sizeMax: 48, durationMin: 4, durationMax: 10, opacityMin: 0.6, opacityMax: 1.0, invisibleChance: 0, swayMin: 1, swayMax: 3 }
    case 'sakura':  return { count: 50, sizeMin: 14, sizeMax: 28, durationMin: 8, durationMax: 20, opacityMin: 0.5, opacityMax: 0.9, invisibleChance: 0, swayMin: 4, swayMax: 14 }
  }
}

function getEmojiSet(effect: Props['effect']): string[] {
  switch (effect) {
    case 'leaves':   return LEAF_EMOJI
    case 'snow':     return SNOW_EMOJI
    case 'fruit':    return FRUIT_EMOJI
    case 'junkfood': return JUNKFOOD_EMOJI
    case 'sakura':   return SAKURA_EMOJI
  }
}

function rand(min: number, max: number) { return min + Math.random() * (max - min) }
function pick<T>(arr: T[]): T { return arr[Math.floor(Math.random() * arr.length)] }

export function CSSParticleEffect({ effect, foreground = false }: Props) {
  const particles = useMemo(() => {
    const cfg = getConfig(effect, foreground)
    const emojis = getEmojiSet(effect)
    const result: EmojiParticle[] = []

    for (let i = 0; i < cfg.count; i++) {
      const invisible = Math.random() < cfg.invisibleChance
      result.push({
        id: i,
        emoji: pick(emojis),
        x: rand(0, 100),
        size: invisible ? rand(cfg.sizeMin, cfg.sizeMax) * 3 : rand(cfg.sizeMin, cfg.sizeMax),
        duration: rand(cfg.durationMin, cfg.durationMax),
        delay: rand(-cfg.durationMax, 0), // negative = already mid-fall on load
        sway: rand(cfg.swayMin, cfg.swayMax),
        startY: rand(-15, -5),
        opacity: invisible ? 0 : rand(cfg.opacityMin, cfg.opacityMax),
        rotation: rand(0, 360),
        rotationEnd: rand(0, 360) * (Math.random() > 0.5 ? 1 : -1),
      })
    }
    return result
  }, [effect, foreground])

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        overflow: 'hidden',
        pointerEvents: 'none',
        zIndex: foreground ? 10 : 0,
      }}
    >
      <style>{`
        @keyframes cssFall {
          from {
            transform: translateY(0) translateX(0) rotate(var(--rot-start));
          }
          25% {
            transform: translateY(30vh) translateX(var(--sway)) rotate(calc(var(--rot-start) + var(--rot-delta) * 0.25));
          }
          50% {
            transform: translateY(55vh) translateX(calc(var(--sway) * -0.6)) rotate(calc(var(--rot-start) + var(--rot-delta) * 0.5));
          }
          75% {
            transform: translateY(80vh) translateX(var(--sway)) rotate(calc(var(--rot-start) + var(--rot-delta) * 0.75));
          }
          to {
            transform: translateY(110vh) translateX(calc(var(--sway) * -0.3)) rotate(calc(var(--rot-start) + var(--rot-delta)));
          }
        }
      `}</style>
      {particles.map(p => (
        <div
          key={p.id}
          style={{
            position: 'absolute',
            left: `${p.x}%`,
            top: `${p.startY}%`,
            fontSize: `${p.size}px`,
            opacity: p.opacity,
            willChange: 'transform',
            animation: `cssFall ${p.duration}s linear ${p.delay}s infinite`,
            '--sway': `${p.sway}vw`,
            '--rot-start': `${p.rotation}deg`,
            '--rot-delta': `${p.rotationEnd}deg`,
            lineHeight: 1,
            userSelect: 'none',
          } as React.CSSProperties}
        >
          {p.emoji}
        </div>
      ))}
    </div>
  )
}
