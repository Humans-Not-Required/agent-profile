import { useMemo } from 'react'

// ── CSS-based emoji particle effects ──────────────────────────────────────────
// GPU-composited via CSS transform animations — zero JS per frame.
// Used for: leaves, snow, fruit, junkfood, sakura, hearts, cactus

interface Props {
  effect: 'leaves' | 'snow' | 'fruit' | 'junkfood' | 'sakura' | 'hearts' | 'cactus'
  foreground?: boolean
}

interface EmojiParticle {
  id: number
  emoji: string
  x: number        // start % from left
  size: number     // font-size in px
  duration: number  // animation duration in seconds
  delay: number     // animation-delay in seconds
  driftX: number    // total horizontal drift in vw (positive = right, negative = left)
  startY: number    // start offset (negative = above viewport)
  opacity: number
  rotation: number  // start rotation deg
  rotationEnd: number // end rotation deg
  hueRotate?: number  // degrees of hue shift (for leaf color variety)
}

const SNOW_EMOJI = ['❄', '❅', '❆']
const FRUIT_EMOJI = ['🍎', '🍊', '🍋', '🍇', '🍓', '🍑', '🍌', '🍉', '🥝', '🍒', '🫐', '🍍']
const JUNKFOOD_EMOJI = [
  '🍕', '🍔', '🌭', '🍟', '🌮', '🌯', '🍗', '🍖', '🥓',
  '🧀', '🍩', '🍪', '🎂', '🍰', '🧁', '🍫', '🍬', '🍭',
  '🥤', '🍦', '🥞', '🧇', '🥨', '🍿', '🥡', '🥪',
]
const SAKURA_EMOJI = ['🌸']
const HEART_EMOJI = ['❤️', '💕', '💖', '💗', '💘', '💝', '🩷', '♥️']
const CACTUS_EMOJI = ['🌵', '🏜️', '☀️', '🦎', '🐪', '🌵', '🌵', '🌵']

// Autumn leaf hue rotations: red(0), deep red(-25), crimson(-40), brown(-55),
// orange(25), gold(45), yellow(60), yellow-green(80), green(120)
const LEAF_HUES = [0, 0, -25, -40, -55, 25, 25, 45, 45, 60, 60, 80, 120]

function getConfig(effect: Props['effect'], foreground: boolean) {
  if (foreground) {
    if (effect === 'snow') {
      return {
        count: 6,
        sizeMin: 120, sizeMax: 260,
        durationMin: 15, durationMax: 30,
        opacityMin: 0.5, opacityMax: 0.8,
        invisibleChance: 0.5,
        driftMin: 3, driftMax: 10,
      }
    }
    return {
      count: 2,
      sizeMin: 40, sizeMax: 80,
      durationMin: 12, durationMax: 25,
      opacityMin: 0, opacityMax: 0.9,
      invisibleChance: 0.85,
      driftMin: 2, driftMax: 8,
    }
  }
  switch (effect) {
    case 'leaves':   return { count: 30, sizeMin: 18, sizeMax: 36, durationMin: 8, durationMax: 18, opacityMin: 0.4, opacityMax: 0.9, invisibleChance: 0, driftMin: 5, driftMax: 20 }
    case 'snow':     return { count: 40, sizeMin: 30, sizeMax: 70, durationMin: 10, durationMax: 25, opacityMin: 1.0, opacityMax: 1.0, invisibleChance: 0, driftMin: 3, driftMax: 10 }
    case 'fruit':    return { count: 50, sizeMin: 28, sizeMax: 56, durationMin: 6, durationMax: 14, opacityMin: 0.5, opacityMax: 1.0, invisibleChance: 0, driftMin: 5, driftMax: 15 }
    case 'junkfood': return { count: 55, sizeMin: 24, sizeMax: 48, durationMin: 4, durationMax: 10, opacityMin: 0.6, opacityMax: 1.0, invisibleChance: 0, driftMin: 3, driftMax: 8 }
    case 'sakura':   return { count: 50, sizeMin: 14, sizeMax: 28, durationMin: 8, durationMax: 20, opacityMin: 0.5, opacityMax: 0.9, invisibleChance: 0, driftMin: 6, driftMax: 18 }
    case 'hearts':   return { count: 35, sizeMin: 20, sizeMax: 48, durationMin: 8, durationMax: 18, opacityMin: 0.6, opacityMax: 1.0, invisibleChance: 0, driftMin: 4, driftMax: 12 }
    case 'cactus':   return { count: 25, sizeMin: 28, sizeMax: 56, durationMin: 12, durationMax: 28, opacityMin: 0.6, opacityMax: 1.0, invisibleChance: 0, driftMin: 3, driftMax: 8 }
  }
}

function getEmojiSet(effect: Props['effect']): string[] {
  switch (effect) {
    case 'leaves':   return ['🍁']  // maple leaf only — color variety via hue-rotate
    case 'snow':     return SNOW_EMOJI
    case 'fruit':    return FRUIT_EMOJI
    case 'junkfood': return JUNKFOOD_EMOJI
    case 'sakura':   return SAKURA_EMOJI
    case 'hearts':   return HEART_EMOJI
    case 'cactus':   return CACTUS_EMOJI
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
      // Random drift direction: positive = right, negative = left
      const driftSign = Math.random() > 0.5 ? 1 : -1
      const driftAmount = rand(cfg.driftMin, cfg.driftMax) * driftSign

      result.push({
        id: i,
        emoji: pick(emojis),
        x: rand(0, 100),
        size: invisible ? rand(cfg.sizeMin, cfg.sizeMax) * 3 : rand(cfg.sizeMin, cfg.sizeMax),
        duration: rand(cfg.durationMin, cfg.durationMax),
        delay: rand(-cfg.durationMax, 0),
        driftX: driftAmount,
        startY: rand(-15, -5),
        opacity: invisible ? 0 : rand(cfg.opacityMin, cfg.opacityMax),
        rotation: rand(0, 360),
        rotationEnd: rand(0, 360) * (Math.random() > 0.5 ? 1 : -1),
        hueRotate: effect === 'leaves' ? pick(LEAF_HUES) : undefined,
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
        @keyframes cssDrift {
          from {
            transform: translateY(0) translateX(0) rotate(var(--rot-start));
          }
          to {
            transform: translateY(115vh) translateX(var(--drift-x)) rotate(calc(var(--rot-start) + var(--rot-delta)));
          }
        }
      `}</style>
      {particles.map(p => {
        const extraStyle: React.CSSProperties = {}

        // Snow: bright glow
        if (effect === 'snow') {
          extraStyle.filter = 'drop-shadow(0 0 6px rgba(180,220,255,0.9)) drop-shadow(0 0 14px rgba(130,190,255,0.6))'
          extraStyle.color = '#e8f4ff'
        }

        // Leaves: autumn color variety via hue-rotate
        if (effect === 'leaves' && p.hueRotate !== undefined && p.hueRotate !== 0) {
          extraStyle.filter = `hue-rotate(${p.hueRotate}deg) saturate(1.2)`
        }

        return (
          <div
            key={p.id}
            style={{
              position: 'absolute',
              left: `${p.x}%`,
              top: `${p.startY}%`,
              fontSize: `${p.size}px`,
              opacity: p.opacity,
              willChange: 'transform',
              animation: `cssDrift ${p.duration}s linear ${p.delay}s infinite`,
              '--drift-x': `${p.driftX}vw`,
              '--rot-start': `${p.rotation}deg`,
              '--rot-delta': `${p.rotationEnd}deg`,
              lineHeight: 1,
              userSelect: 'none',
              ...extraStyle,
            } as React.CSSProperties}
          >
            {p.emoji}
          </div>
        )
      })}
    </div>
  )
}
