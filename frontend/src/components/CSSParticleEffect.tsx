import { useMemo } from 'react'

// ── CSS-based emoji particle effects ──────────────────────────────────────────
// GPU-composited via CSS transform animations — zero JS per frame.
// Used for: leaves, snow, fruit, junkfood, sakura, hearts, cactus
//
// NOTE: CSS custom properties (var()) inside @keyframes are "animation-tainted"
// and don't work reliably across browsers (Safari, older Chrome, mobile).
// Instead, we generate per-particle keyframes with values baked in.

interface Props {
  effect: 'leaves' | 'snow' | 'fruit' | 'junkfood' | 'sakura' | 'hearts' | 'cactus'
  foreground?: boolean
}

interface EmojiParticle {
  id: number
  emoji: string
  x: number
  size: number
  duration: number
  delay: number
  driftX: number      // vw
  startY: number      // % from top (negative = above viewport)
  opacity: number
  rotStart: number    // deg
  rotEnd: number      // deg (total, not delta)
  hueRotate?: number
  wanderDuration: number  // slow horizontal wander cycle (seconds)
  wanderRange: number     // vw range for horizontal wander
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

const LEAF_HUES = [0, 0, -25, -40, -55, 25, 25, 45, 45, 60, 60, 80, 120]

function getConfig(effect: Props['effect'], foreground: boolean) {
  if (foreground) {
    if (effect === 'snow') {
      return { count: 2, sizeMin: 70, sizeMax: 130, durationMin: 10, durationMax: 18, opacityMin: 0.3, opacityMax: 0.5, invisibleChance: 0.4, driftMin: 3, driftMax: 10 }
    }
    return { count: 2, sizeMin: 40, sizeMax: 80, durationMin: 12, durationMax: 25, opacityMin: 0, opacityMax: 0.9, invisibleChance: 0.85, driftMin: 2, driftMax: 8 }
  }
  switch (effect) {
    case 'leaves':   return { count: 30, sizeMin: 18, sizeMax: 36, durationMin: 8, durationMax: 18, opacityMin: 0.4, opacityMax: 0.9, invisibleChance: 0, driftMin: 5, driftMax: 20 }
    case 'snow':     return { count: 45, sizeMin: 16, sizeMax: 42, durationMin: 5, durationMax: 14, opacityMin: 0.8, opacityMax: 1.0, invisibleChance: 0, driftMin: 3, driftMax: 10 }
    case 'fruit':    return { count: 50, sizeMin: 28, sizeMax: 56, durationMin: 6, durationMax: 14, opacityMin: 0.5, opacityMax: 1.0, invisibleChance: 0, driftMin: 5, driftMax: 15 }
    case 'junkfood': return { count: 55, sizeMin: 24, sizeMax: 48, durationMin: 4, durationMax: 10, opacityMin: 0.6, opacityMax: 1.0, invisibleChance: 0, driftMin: 3, driftMax: 8 }
    case 'sakura':   return { count: 50, sizeMin: 14, sizeMax: 28, durationMin: 8, durationMax: 20, opacityMin: 0.5, opacityMax: 0.9, invisibleChance: 0, driftMin: 6, driftMax: 18 }
    case 'hearts':   return { count: 35, sizeMin: 20, sizeMax: 48, durationMin: 8, durationMax: 18, opacityMin: 0.6, opacityMax: 1.0, invisibleChance: 0, driftMin: 4, driftMax: 12 }
    case 'cactus':   return { count: 25, sizeMin: 28, sizeMax: 56, durationMin: 12, durationMax: 28, opacityMin: 0.6, opacityMax: 1.0, invisibleChance: 0, driftMin: 3, driftMax: 8 }
  }
}

function getEmojiSet(effect: Props['effect']): string[] {
  switch (effect) {
    case 'leaves':   return ['🍁']
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
  const { particles, keyframesCSS } = useMemo(() => {
    const cfg = getConfig(effect, foreground)
    const emojis = getEmojiSet(effect)
    const result: EmojiParticle[] = []
    const kfParts: string[] = []

    for (let i = 0; i < cfg.count; i++) {
      const invisible = Math.random() < cfg.invisibleChance
      const driftSign = Math.random() > 0.5 ? 1 : -1
      const driftX = rand(cfg.driftMin, cfg.driftMax) * driftSign
      const rotStart = rand(0, 360)
      const rotEnd = rotStart + rand(0, 360) * (Math.random() > 0.5 ? 1 : -1)

      // Slow horizontal wander — prime-ish duration so it never syncs with the fall
      const wanderDuration = rand(37, 73)  // 37-73s, long and varied
      const wanderRange = rand(10, 35)     // wanders ±10-35vw over time

      const p: EmojiParticle = {
        id: i,
        emoji: pick(emojis),
        x: rand(0, 100),
        size: invisible ? rand(cfg.sizeMin, cfg.sizeMax) * 3 : rand(cfg.sizeMin, cfg.sizeMax),
        duration: rand(cfg.durationMin, cfg.durationMax),
        delay: rand(-cfg.durationMax, 0),
        driftX,
        startY: rand(-15, -5),
        opacity: invisible ? 0 : rand(cfg.opacityMin, cfg.opacityMax),
        rotStart,
        rotEnd,
        hueRotate: effect === 'leaves' ? pick(LEAF_HUES) : undefined,
        wanderDuration,
        wanderRange,
      }
      result.push(p)

      // Per-particle fall keyframe (on inner element)
      const fallName = `f${effect[0]}${i}`
      kfParts.push(
        `@keyframes ${fallName}{from{transform:translateY(0) translateX(0) rotate(${rotStart}deg)}to{transform:translateY(115vh) translateX(${driftX}vw) rotate(${rotEnd}deg)}}`
      )
      // Per-particle horizontal wander keyframe (on outer wrapper — long non-aligned cycle)
      const wanderName = `w${effect[0]}${i}`
      const wanderSign = Math.random() > 0.5 ? 1 : -1
      kfParts.push(
        `@keyframes ${wanderName}{0%{transform:translateX(0)}33%{transform:translateX(${wanderRange * wanderSign}vw)}66%{transform:translateX(${-wanderRange * wanderSign * 0.6}vw)}100%{transform:translateX(0)}}`
      )
    }

    return { particles: result, keyframesCSS: kfParts.join('\n') }
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
      <style>{keyframesCSS}</style>
      {particles.map(p => {
        const extraStyle: React.CSSProperties = {}

        if (effect === 'snow') {
          extraStyle.filter = 'drop-shadow(0 0 6px rgba(180,220,255,0.9)) drop-shadow(0 0 14px rgba(130,190,255,0.6))'
          extraStyle.color = '#e8f4ff'
        }

        if (effect === 'leaves' && p.hueRotate !== undefined && p.hueRotate !== 0) {
          extraStyle.filter = `hue-rotate(${p.hueRotate}deg) saturate(1.2)`
        }

        const fallName = `f${effect[0]}${p.id}`
        const wanderName = `w${effect[0]}${p.id}`

        return (
          <div
            key={p.id}
            style={{
              position: 'absolute',
              left: `${p.x}%`,
              top: `${p.startY}%`,
              willChange: 'transform',
              animation: `${wanderName} ${p.wanderDuration}s ease-in-out ${p.delay}s infinite`,
            }}
          >
            <div
              style={{
                fontSize: `${p.size}px`,
                opacity: p.opacity,
                willChange: 'transform',
                animation: `${fallName} ${p.duration}s linear ${p.delay}s infinite`,
                lineHeight: 1,
                userSelect: 'none',
                ...extraStyle,
              } as React.CSSProperties}
            >
              {p.emoji}
            </div>
          </div>
        )
      })}
    </div>
  )
}
