import { useEffect, useRef } from 'react'

export type EffectName = 'snow' | 'leaves' | 'rain' | 'fireflies' | 'stars' | 'sakura' | 'embers' | 'digital-rain' | 'none'

interface Props {
  effect: EffectName
  enabled: boolean
  seasonal: boolean
}

/** Returns the seasonal effect name based on current UTC month. */
function seasonalEffect(): EffectName {
  const month = new Date().getUTCMonth() + 1 // 1-12
  if (month >= 12 || month <= 2) return 'snow'      // Winter: Dec-Feb
  if (month >= 3 && month <= 5)  return 'stars'     // Spring: Mar-May
  if (month >= 6 && month <= 8)  return 'fireflies' // Summer: Jun-Aug
  return 'leaves'                                    // Autumn: Sep-Nov
}

// ── Particle definitions ──────────────────────────────────────────────────────

interface Particle {
  x: number
  y: number
  vx: number
  vy: number
  size: number
  opacity: number
  rotation?: number
  vr?: number       // rotational velocity
  phase?: number    // for fireflies/stars twinkle
}

function initParticles(count: number, w: number, h: number): Particle[] {
  return Array.from({ length: count }, () => ({
    x: Math.random() * w,
    y: Math.random() * h,
    vx: (Math.random() - 0.5) * 0.5,
    vy: Math.random() * 1 + 0.3,
    size: Math.random() * 4 + 2,
    opacity: Math.random() * 0.7 + 0.3,
    rotation: Math.random() * Math.PI * 2,
    vr: (Math.random() - 0.5) * 0.05,
    phase: Math.random() * Math.PI * 2,
  }))
}

function drawSnow(ctx: CanvasRenderingContext2D, p: Particle) {
  ctx.beginPath()
  ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2)
  ctx.fillStyle = `rgba(220, 235, 255, ${p.opacity})`
  ctx.fill()
}

function drawLeaf(ctx: CanvasRenderingContext2D, p: Particle) {
  ctx.save()
  ctx.translate(p.x, p.y)
  ctx.rotate(p.rotation ?? 0)
  ctx.beginPath()
  ctx.ellipse(0, 0, p.size * 1.5, p.size * 0.6, 0, 0, Math.PI * 2)
  ctx.fillStyle = `rgba(100, 180, 80, ${p.opacity})`
  ctx.fill()
  ctx.restore()
}

function drawRain(ctx: CanvasRenderingContext2D, p: Particle, w: number) {
  ctx.beginPath()
  ctx.moveTo(p.x, p.y)
  ctx.lineTo(p.x + w * 0.01, p.y + p.size * 4)
  ctx.strokeStyle = `rgba(150, 190, 230, ${p.opacity * 0.6})`
  ctx.lineWidth = 1
  ctx.stroke()
}

function drawFirefly(ctx: CanvasRenderingContext2D, p: Particle, t: number) {
  const pulse = Math.sin((p.phase ?? 0) + t * 0.003) * 0.4 + 0.6
  const r = p.size * pulse
  const gradient = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, r * 3)
  gradient.addColorStop(0, `rgba(200, 255, 120, ${p.opacity * pulse})`)
  gradient.addColorStop(1, 'rgba(200, 255, 120, 0)')
  ctx.beginPath()
  ctx.arc(p.x, p.y, r * 3, 0, Math.PI * 2)
  ctx.fillStyle = gradient
  ctx.fill()
}

function drawStar(ctx: CanvasRenderingContext2D, p: Particle, t: number) {
  const twinkle = Math.sin((p.phase ?? 0) + t * 0.002) * 0.3 + 0.7
  ctx.beginPath()
  ctx.arc(p.x, p.y, p.size * 0.7, 0, Math.PI * 2)
  ctx.fillStyle = `rgba(220, 230, 255, ${p.opacity * twinkle})`
  ctx.fill()
}

function drawSakura(ctx: CanvasRenderingContext2D, p: Particle) {
  ctx.save()
  ctx.translate(p.x, p.y)
  ctx.rotate(p.rotation ?? 0)
  // 5-petal flower approximation with overlapping ellipses
  for (let i = 0; i < 5; i++) {
    ctx.save()
    ctx.rotate((i * Math.PI * 2) / 5)
    ctx.beginPath()
    ctx.ellipse(p.size * 0.8, 0, p.size * 0.9, p.size * 0.5, 0, 0, Math.PI * 2)
    ctx.fillStyle = `rgba(255, 182, 193, ${p.opacity * 0.85})`
    ctx.fill()
    ctx.restore()
  }
  ctx.restore()
}

function drawEmber(ctx: CanvasRenderingContext2D, p: Particle, t: number) {
  // Glowing ember/spark that drifts upward with flickering intensity
  const flicker = Math.sin((p.phase ?? 0) + t * 0.006) * 0.3 + 0.7
  const r = p.size * 0.6 * flicker
  const gradient = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, r * 4)
  // Core: bright white-yellow, halo: orange-red
  gradient.addColorStop(0, `rgba(255, 200, 80, ${p.opacity * flicker})`)
  gradient.addColorStop(0.3, `rgba(255, 100, 20, ${p.opacity * flicker * 0.7})`)
  gradient.addColorStop(1, 'rgba(200, 40, 0, 0)')
  ctx.beginPath()
  ctx.arc(p.x, p.y, r * 4, 0, Math.PI * 2)
  ctx.fillStyle = gradient
  ctx.fill()
  // Bright core dot
  ctx.beginPath()
  ctx.arc(p.x, p.y, r * 0.5, 0, Math.PI * 2)
  ctx.fillStyle = `rgba(255, 220, 140, ${p.opacity * flicker})`
  ctx.fill()
}

// ── Digital rain (Matrix) helpers ──
const MATRIX_CHARS = 'アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン0123456789ABCDEF'

interface RainColumn {
  x: number
  chars: string[]
  y: number         // head position
  speed: number
  length: number
  charSize: number
}

function initRainColumns(w: number, h: number): RainColumn[] {
  const charSize = 14
  const cols = Math.floor(w / (charSize * 0.85))
  return Array.from({ length: cols }, (_, i) => ({
    x: i * (charSize * 0.85) + charSize * 0.4,
    chars: Array.from({ length: Math.floor(h / charSize) + 10 }, () =>
      MATRIX_CHARS[Math.floor(Math.random() * MATRIX_CHARS.length)]
    ),
    y: Math.random() * h * 2 - h,      // start at random offset
    speed: Math.random() * 2 + 1.5,
    length: Math.floor(Math.random() * 12) + 8,
    charSize,
  }))
}

function drawDigitalRain(ctx: CanvasRenderingContext2D, columns: RainColumn[], h: number, t: number) {
  for (const col of columns) {
    const headY = col.y
    for (let i = 0; i < col.length; i++) {
      const cy = headY - i * col.charSize
      if (cy < -col.charSize || cy > h + col.charSize) continue
      const charIdx = (Math.floor(cy / col.charSize) + col.chars.length * 10) % col.chars.length

      // Occasionally randomize character for flicker
      let ch = col.chars[charIdx]
      if (Math.random() < 0.01) {
        ch = MATRIX_CHARS[Math.floor(Math.random() * MATRIX_CHARS.length)]
        col.chars[charIdx] = ch
      }

      if (i === 0) {
        // Head character: bright white-green
        ctx.fillStyle = 'rgba(180, 255, 180, 0.95)'
        ctx.font = `bold ${col.charSize}px "Courier New", monospace`
      } else {
        // Trail: fade from bright green to dark green
        const fade = 1 - (i / col.length)
        const g = Math.floor(180 * fade + 40)
        ctx.fillStyle = `rgba(0, ${g}, 0, ${fade * 0.8})`
        ctx.font = `${col.charSize}px "Courier New", monospace`
      }
      ctx.fillText(ch, col.x, cy)
    }

    // Advance column
    col.y += col.speed
    if (col.y - col.length * col.charSize > h) {
      col.y = -col.length * col.charSize * Math.random()
      col.speed = Math.random() * 2 + 1.5
      col.length = Math.floor(Math.random() * 12) + 8
    }
  }
}

// ── Main component ─────────────────────────────────────────────────────────────

export function ParticleEffect({ effect, enabled, seasonal }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const rafRef = useRef<number>(0)

  // Resolve actual effect (seasonal override)
  const activeEffect: EffectName = !enabled
    ? 'none'
    : seasonal
      ? seasonalEffect()
      : effect

  useEffect(() => {
    if (activeEffect === 'none') return

    const canvas = canvasRef.current
    if (!canvas) return
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    // Resize canvas to viewport
    const resize = () => {
      canvas.width = window.innerWidth
      canvas.height = window.innerHeight
    }
    resize()
    window.addEventListener('resize', resize)

    // Digital rain uses its own column-based system
    let rainColumns: RainColumn[] = []
    if (activeEffect === 'digital-rain') {
      rainColumns = initRainColumns(canvas.width, canvas.height)
    }

    // Particle count by effect type
    const countMap: Record<EffectName, number> = {
      snow: 100, leaves: 60, rain: 150, fireflies: 50, stars: 200, sakura: 60,
      embers: 80, 'digital-rain': 0, none: 0,
    }
    const count = countMap[activeEffect] ?? 80
    const particles = count > 0 ? initParticles(count, canvas.width, canvas.height) : []

    // Embers drift upward — invert their initial velocity
    if (activeEffect === 'embers') {
      for (const p of particles) {
        p.vy = -(Math.random() * 0.8 + 0.2) // drift up
        p.vx = (Math.random() - 0.5) * 0.6   // gentle sway
        p.size = Math.random() * 3 + 1        // smaller particles
      }
    }

    let t = 0
    const animate = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height)
      t++

      // Digital rain has its own rendering path
      if (activeEffect === 'digital-rain') {
        drawDigitalRain(ctx, rainColumns, canvas.height, t)
        rafRef.current = requestAnimationFrame(animate)
        return
      }

      for (const p of particles) {
        // Draw
        switch (activeEffect) {
          case 'snow':      drawSnow(ctx, p); break
          case 'leaves':    drawLeaf(ctx, p); break
          case 'rain':      drawRain(ctx, p, canvas.width); break
          case 'fireflies': drawFirefly(ctx, p, t); break
          case 'stars':     drawStar(ctx, p, t); break
          case 'sakura':    drawSakura(ctx, p); break
          case 'embers':    drawEmber(ctx, p, t); break
        }

        // Move
        if (activeEffect === 'fireflies') {
          // Fireflies drift gently
          p.x += Math.sin((p.phase ?? 0) + t * 0.01) * 0.5
          p.y += Math.sin((p.phase ?? 0) * 1.3 + t * 0.008) * 0.4
        } else if (activeEffect === 'stars') {
          // Stars barely move — just twinkle
          p.x += p.vx * 0.05
          p.y += p.vy * 0.05
        } else if (activeEffect === 'rain') {
          p.x += 1.5  // diagonal slant
          p.y += 12
        } else if (activeEffect === 'embers') {
          // Embers drift upward with swaying
          p.x += p.vx + Math.sin((p.phase ?? 0) + t * 0.015) * 0.3
          p.y += p.vy
          // Fade out as they rise
          p.opacity -= 0.001
          if (p.opacity < 0.05) {
            p.opacity = Math.random() * 0.7 + 0.3
            p.y = canvas.height + 10
            p.x = Math.random() * canvas.width
          }
        } else {
          p.x += p.vx
          p.y += p.vy
          if (p.rotation !== undefined && p.vr !== undefined) p.rotation += p.vr
        }

        // Wrap around edges
        const w = canvas.width
        const h = canvas.height
        if (p.y > h + 20)  p.y = -20
        if (p.y < -20)     p.y = h + 20
        if (p.x > w + 20)  p.x = -20
        if (p.x < -20)     p.x = w + 20
      }

      rafRef.current = requestAnimationFrame(animate)
    }

    rafRef.current = requestAnimationFrame(animate)

    return () => {
      cancelAnimationFrame(rafRef.current)
      window.removeEventListener('resize', resize)
    }
  }, [activeEffect])

  if (activeEffect === 'none') return null

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: 'fixed',
        top: 0, left: 0,
        width: '100vw', height: '100vh',
        pointerEvents: 'none',
        zIndex: 0,
      }}
    />
  )
}
