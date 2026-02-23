import { useEffect, useRef } from 'react'

export type EffectName = 'snow' | 'leaves' | 'rain' | 'fireflies' | 'stars' | 'sakura' | 'embers' | 'digital-rain' | 'flames' | 'none'

interface Props {
  effect: EffectName
  enabled: boolean
  seasonal: boolean
  foreground?: boolean  // if true, render fewer particles above content
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
  layer?: number    // depth layer for starfield (0=far, 1=mid, 2=near)
  color?: number    // color variant index
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
    layer: Math.floor(Math.random() * 3),
    color: Math.floor(Math.random() * 5),
  }))
}

// ── Snowflake: 6-armed crystal with branches ──

function drawSnowflake(ctx: CanvasRenderingContext2D, p: Particle) {
  ctx.save()
  ctx.translate(p.x, p.y)
  ctx.rotate(p.rotation ?? 0)
  ctx.strokeStyle = `rgba(210, 230, 255, ${p.opacity})`
  ctx.lineWidth = Math.max(0.5, p.size * 0.15)
  ctx.lineCap = 'round'

  const r = p.size * 1.8
  const branchLen = r * 0.35

  for (let i = 0; i < 6; i++) {
    const angle = (i * Math.PI) / 3
    const cos = Math.cos(angle)
    const sin = Math.sin(angle)

    // Main arm
    ctx.beginPath()
    ctx.moveTo(0, 0)
    ctx.lineTo(cos * r, sin * r)
    ctx.stroke()

    // Two branches on each arm at ~60% length
    const bx = cos * r * 0.6
    const by = sin * r * 0.6
    const bAngle1 = angle + Math.PI / 6
    const bAngle2 = angle - Math.PI / 6
    ctx.beginPath()
    ctx.moveTo(bx, by)
    ctx.lineTo(bx + Math.cos(bAngle1) * branchLen, by + Math.sin(bAngle1) * branchLen)
    ctx.stroke()
    ctx.beginPath()
    ctx.moveTo(bx, by)
    ctx.lineTo(bx + Math.cos(bAngle2) * branchLen, by + Math.sin(bAngle2) * branchLen)
    ctx.stroke()
  }

  // Center dot
  ctx.beginPath()
  ctx.arc(0, 0, p.size * 0.2, 0, Math.PI * 2)
  ctx.fillStyle = `rgba(230, 240, 255, ${p.opacity * 0.8})`
  ctx.fill()

  ctx.restore()
}

// ── Leaf ──

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

// ── Rain ──

function drawRain(ctx: CanvasRenderingContext2D, p: Particle, w: number) {
  ctx.beginPath()
  ctx.moveTo(p.x, p.y)
  ctx.lineTo(p.x + w * 0.01, p.y + p.size * 4)
  ctx.strokeStyle = `rgba(150, 190, 230, ${p.opacity * 0.6})`
  ctx.lineWidth = 1
  ctx.stroke()
}

// ── Firefly ──

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

// ── 3D Starfield: perspective projection warp-speed, transparent canvas ──

interface Star3D {
  x: number   // 3D position
  y: number
  z: number   // depth (1..maxZ)
  color: number  // color temperature index
}

const STAR_MAX_Z = 1000
const STAR_FOCAL = 128     // focal length — controls FOV
const STAR_SPEED = 3       // z units per frame (background)
const STAR_SPEED_FG = 1    // slower for foreground layer

function initStars3D(count: number, w: number, h: number): Star3D[] {
  // Spread proportional to viewport so stars cover the full screen at all depths
  const spreadX = w * (STAR_MAX_Z / STAR_FOCAL) * 0.6
  const spreadY = h * (STAR_MAX_Z / STAR_FOCAL) * 0.6
  return Array.from({ length: count }, () => ({
    x: Math.random() * spreadX * 2 - spreadX,
    y: Math.random() * spreadY * 2 - spreadY,
    z: Math.random() * STAR_MAX_Z + 1,
    color: Math.floor(Math.random() * 5),
  }))
}

function resetStar(s: Star3D, w: number, h: number) {
  const spreadX = w * (STAR_MAX_Z / STAR_FOCAL) * 0.6
  const spreadY = h * (STAR_MAX_Z / STAR_FOCAL) * 0.6
  s.x = Math.random() * spreadX * 2 - spreadX
  s.y = Math.random() * spreadY * 2 - spreadY
  s.z = STAR_MAX_Z + Math.random() * 200
  s.color = Math.floor(Math.random() * 5)
}

function drawStarfield3D(
  ctx: CanvasRenderingContext2D,
  stars: Star3D[],
  cx: number, cy: number,
  w: number, h: number,
  speed: number,
) {
  // Transparent clear — page theme background shows through
  ctx.clearRect(0, 0, w, h)

  const colors = [
    [220, 230, 255],  // cool white
    [170, 200, 255],  // blue-white
    [255, 240, 210],  // warm white
    [190, 215, 255],  // pale blue
    [255, 215, 185],  // warm orange
  ]

  for (const s of stars) {
    // Compute current screen position
    const k = STAR_FOCAL / Math.max(s.z, 1)
    const sx = s.x * k + cx
    const sy = s.y * k + cy

    // Move toward camera
    s.z -= speed

    // Compute new screen position after move (for streak direction)
    const k2 = STAR_FOCAL / Math.max(s.z, 1)
    const sx2 = s.x * k2 + cx
    const sy2 = s.y * k2 + cy

    // Reset if past camera or way off screen
    if (s.z < 1) {
      resetStar(s, w, h)
      continue
    }

    // Skip if off screen
    if (sx2 < -50 || sx2 > w + 50 || sy2 < -50 || sy2 > h + 50) continue

    // Depth ratio: 0 = far, 1 = close
    const depthRatio = 1 - s.z / STAR_MAX_Z
    const size = Math.max(0.3, depthRatio * 3.5)
    const brightness = 0.15 + depthRatio * 0.85

    const [cr, cg, cb] = colors[s.color]
    const r = Math.floor(cr * brightness)
    const g = Math.floor(cg * brightness)
    const b = Math.floor(cb * brightness)

    // Streak line — from previous position to current, showing motion direction
    const dx = sx2 - sx
    const dy = sy2 - sy
    const streakLen = Math.sqrt(dx * dx + dy * dy)

    if (streakLen > 0.8 && depthRatio > 0.1) {
      // Extend streak backwards from current position for visible trail
      const extend = Math.min(streakLen * 3, 40 * depthRatio)
      const nx = dx / streakLen  // normalized direction
      const ny = dy / streakLen
      const tailX = sx2 - nx * extend
      const tailY = sy2 - ny * extend

      ctx.strokeStyle = `rgba(${r}, ${g}, ${b}, ${brightness * 0.5})`
      ctx.lineWidth = Math.max(0.3, size * 0.5)
      ctx.beginPath()
      ctx.moveTo(tailX, tailY)
      ctx.lineTo(sx2, sy2)
      ctx.stroke()
    }

    // Star dot
    ctx.beginPath()
    ctx.arc(sx2, sy2, size, 0, Math.PI * 2)
    ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${brightness})`
    ctx.fill()

    // Glow on close stars
    if (depthRatio > 0.55) {
      const glowR = size * 4
      const gradient = ctx.createRadialGradient(sx2, sy2, 0, sx2, sy2, glowR)
      gradient.addColorStop(0, `rgba(${r}, ${g}, ${b}, ${brightness * 0.2})`)
      gradient.addColorStop(1, `rgba(${r}, ${g}, ${b}, 0)`)
      ctx.beginPath()
      ctx.arc(sx2, sy2, glowR, 0, Math.PI * 2)
      ctx.fillStyle = gradient
      ctx.fill()
    }
  }
}

// ── Sakura: teardrop petals with pink gradient ──

function drawSakuraPetal(ctx: CanvasRenderingContext2D, p: Particle) {
  ctx.save()
  ctx.translate(p.x, p.y)
  ctx.rotate(p.rotation ?? 0)

  const s = p.size * 1.2

  // Draw 5 teardrop-shaped petals
  for (let i = 0; i < 5; i++) {
    ctx.save()
    ctx.rotate((i * Math.PI * 2) / 5)

    ctx.beginPath()
    // Teardrop: start narrow at center, widen, then round tip
    ctx.moveTo(0, 0)
    ctx.bezierCurveTo(
      s * 0.3, -s * 0.3,   // control point 1 (left curve out)
      s * 1.0, -s * 0.2,   // control point 2 (tip area)
      s * 0.9, 0            // tip
    )
    ctx.bezierCurveTo(
      s * 1.0, s * 0.2,    // control point 3 (right of tip)
      s * 0.3, s * 0.3,    // control point 4 (right curve back)
      0, 0                  // back to center
    )

    // Gradient from deep pink at center to lighter pink at tip
    const grad = ctx.createLinearGradient(0, 0, s * 0.9, 0)
    grad.addColorStop(0, `rgba(220, 130, 160, ${p.opacity * 0.9})`)
    grad.addColorStop(0.5, `rgba(255, 182, 200, ${p.opacity * 0.85})`)
    grad.addColorStop(1, `rgba(255, 210, 220, ${p.opacity * 0.7})`)
    ctx.fillStyle = grad
    ctx.fill()

    ctx.restore()
  }

  // Center: small yellow-pink dot (pistil)
  ctx.beginPath()
  ctx.arc(0, 0, s * 0.15, 0, Math.PI * 2)
  ctx.fillStyle = `rgba(255, 200, 150, ${p.opacity * 0.9})`
  ctx.fill()

  ctx.restore()
}

// ── Ember ──

function drawEmber(ctx: CanvasRenderingContext2D, p: Particle, t: number) {
  const flicker = Math.sin((p.phase ?? 0) + t * 0.006) * 0.3 + 0.7
  const r = p.size * 0.6 * flicker
  const gradient = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, r * 4)
  gradient.addColorStop(0, `rgba(255, 200, 80, ${p.opacity * flicker})`)
  gradient.addColorStop(0.3, `rgba(255, 100, 20, ${p.opacity * flicker * 0.7})`)
  gradient.addColorStop(1, 'rgba(200, 40, 0, 0)')
  ctx.beginPath()
  ctx.arc(p.x, p.y, r * 4, 0, Math.PI * 2)
  ctx.fillStyle = gradient
  ctx.fill()
  ctx.beginPath()
  ctx.arc(p.x, p.y, r * 0.5, 0, Math.PI * 2)
  ctx.fillStyle = `rgba(255, 220, 140, ${p.opacity * flicker})`
  ctx.fill()
}

// ── Flames: bezier tongue shapes rising from bottom with base glow ──

function drawFlames(ctx: CanvasRenderingContext2D, particles: Particle[], t: number, w: number, h: number) {
  // 1) Base glow along the bottom — warm ambient light
  const baseH = h * 0.15
  const baseGrad = ctx.createLinearGradient(0, h, 0, h - baseH)
  const pulse = Math.sin(t * 0.02) * 0.05 + 0.95
  baseGrad.addColorStop(0, `rgba(255, 120, 0, ${0.25 * pulse})`)
  baseGrad.addColorStop(0.3, `rgba(255, 60, 0, ${0.12 * pulse})`)
  baseGrad.addColorStop(0.7, `rgba(180, 20, 0, ${0.04 * pulse})`)
  baseGrad.addColorStop(1, 'rgba(100, 0, 0, 0)')
  ctx.fillStyle = baseGrad
  ctx.fillRect(0, h - baseH, w, baseH)

  // 2) Draw each flame tongue as a bezier shape
  for (const p of particles) {
    const life = (h - p.y) / (h * 0.6)  // 0=bottom, 1=risen 60% of screen
    if (life < 0 || life > 1) continue

    const flicker = Math.sin((p.phase ?? 0) + t * 0.01) * 0.15 + 0.85
    const wobble = Math.sin((p.phase ?? 0) * 1.7 + t * 0.008) * (12 + p.size * 3)

    // Tongue width narrows as it rises
    const tongueW = (p.size * 3 + 8) * (1 - life * 0.7) * flicker
    // Tongue height from its base position
    const tongueH = (p.size * 12 + 40) * flicker

    const cx = p.x + wobble
    const baseY = p.y + tongueH * 0.5  // bottom of tongue
    const tipY = p.y - tongueH * 0.5   // top tip

    // Opacity: bright at base, fades at tip
    const fadeOp = p.opacity * (1 - life * life) * flicker
    if (fadeOp < 0.01) continue

    ctx.save()

    // Draw tongue shape with bezier curves
    ctx.beginPath()
    ctx.moveTo(cx, baseY)
    // Left edge curves out then in to tip
    ctx.bezierCurveTo(
      cx - tongueW * 0.8, baseY - tongueH * 0.2,
      cx - tongueW * 0.5, tipY + tongueH * 0.3,
      cx + wobble * 0.1, tipY
    )
    // Right edge mirrors back
    ctx.bezierCurveTo(
      cx + tongueW * 0.5, tipY + tongueH * 0.3,
      cx + tongueW * 0.8, baseY - tongueH * 0.2,
      cx, baseY
    )

    // Vertical gradient per tongue: yellow-white base → orange mid → red tip
    const tongueGrad = ctx.createLinearGradient(cx, baseY, cx, tipY)
    tongueGrad.addColorStop(0, `rgba(255, 240, 160, ${fadeOp * 0.9})`)
    tongueGrad.addColorStop(0.15, `rgba(255, 200, 60, ${fadeOp * 0.85})`)
    tongueGrad.addColorStop(0.4, `rgba(255, 120, 10, ${fadeOp * 0.7})`)
    tongueGrad.addColorStop(0.7, `rgba(220, 50, 0, ${fadeOp * 0.4})`)
    tongueGrad.addColorStop(1, `rgba(120, 10, 0, ${fadeOp * 0.05})`)

    ctx.fillStyle = tongueGrad
    ctx.fill()

    // Inner bright core — narrower, brighter
    if (p.size > 3) {
      ctx.beginPath()
      const coreW = tongueW * 0.3
      const coreH = tongueH * 0.6
      const coreBase = baseY
      const coreTip = baseY - coreH
      ctx.moveTo(cx, coreBase)
      ctx.bezierCurveTo(
        cx - coreW, coreBase - coreH * 0.3,
        cx - coreW * 0.5, coreTip + coreH * 0.2,
        cx, coreTip
      )
      ctx.bezierCurveTo(
        cx + coreW * 0.5, coreTip + coreH * 0.2,
        cx + coreW, coreBase - coreH * 0.3,
        cx, coreBase
      )
      const coreGrad = ctx.createLinearGradient(cx, coreBase, cx, coreTip)
      coreGrad.addColorStop(0, `rgba(255, 255, 240, ${fadeOp * 0.6})`)
      coreGrad.addColorStop(0.5, `rgba(255, 230, 120, ${fadeOp * 0.3})`)
      coreGrad.addColorStop(1, 'rgba(255, 180, 60, 0)')
      ctx.fillStyle = coreGrad
      ctx.fill()
    }

    ctx.restore()
  }
}

// ── Digital rain (Matrix) helpers ──
const MATRIX_CHARS = 'アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン0123456789ABCDEF'

interface RainColumn {
  x: number
  chars: string[]
  y: number
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
    y: Math.random() * h * 2 - h,
    speed: Math.random() * 2 + 1.5,
    length: Math.floor(Math.random() * 12) + 8,
    charSize,
  }))
}

function drawDigitalRain(ctx: CanvasRenderingContext2D, columns: RainColumn[], h: number, _t: number) {
  for (const col of columns) {
    const headY = col.y
    for (let i = 0; i < col.length; i++) {
      const cy = headY - i * col.charSize
      if (cy < -col.charSize || cy > h + col.charSize) continue
      const charIdx = (Math.floor(cy / col.charSize) + col.chars.length * 10) % col.chars.length

      let ch = col.chars[charIdx]
      if (Math.random() < 0.01) {
        ch = MATRIX_CHARS[Math.floor(Math.random() * MATRIX_CHARS.length)]
        col.chars[charIdx] = ch
      }

      if (i === 0) {
        ctx.fillStyle = 'rgba(180, 255, 180, 0.95)'
        ctx.font = `bold ${col.charSize}px "Courier New", monospace`
      } else {
        const fade = 1 - (i / col.length)
        const g = Math.floor(180 * fade + 40)
        ctx.fillStyle = `rgba(0, ${g}, 0, ${fade * 0.8})`
        ctx.font = `${col.charSize}px "Courier New", monospace`
      }
      ctx.fillText(ch, col.x, cy)
    }

    col.y += col.speed
    if (col.y - col.length * col.charSize > h) {
      col.y = -col.length * col.charSize * Math.random()
      col.speed = Math.random() * 2 + 1.5
      col.length = Math.floor(Math.random() * 12) + 8
    }
  }
}

// ── Main component ─────────────────────────────────────────────────────────────

export function ParticleEffect({ effect, enabled, seasonal, foreground = false }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const rafRef = useRef<number>(0)

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

    const resize = () => {
      canvas.width = window.innerWidth
      canvas.height = window.innerHeight
    }
    resize()
    window.addEventListener('resize', resize)

    let rainColumns: RainColumn[] = []
    if (activeEffect === 'digital-rain') {
      rainColumns = initRainColumns(canvas.width, canvas.height)
    }

    // 3D starfield has its own system
    let stars3D: Star3D[] = []
    if (activeEffect === 'stars') {
      const starCount = foreground ? 80 : 800
      stars3D = initStars3D(starCount, canvas.width, canvas.height)
    }

    // Background counts — generous for immersion
    const bgCountMap: Record<EffectName, number> = {
      snow: 160, leaves: 100, rain: 250, fireflies: 90, stars: 500, sakura: 80,
      embers: 140, 'digital-rain': 0, flames: 200, none: 0,
    }
    // Foreground: ~15% of background for subtle depth
    const fgCountMap: Record<EffectName, number> = {
      snow: 12, leaves: 8, rain: 20, fireflies: 6, stars: 25, sakura: 6,
      embers: 10, 'digital-rain': 0, flames: 15, none: 0,
    }
    const countMap = foreground ? fgCountMap : bgCountMap
    const count = countMap[activeEffect] ?? 80
    const particles = count > 0 ? initParticles(count, canvas.width, canvas.height) : []

    // Foreground particles: larger + slightly more opaque for depth
    if (foreground) {
      for (const p of particles) {
        p.size *= 1.5
        p.opacity = Math.min(1, p.opacity * 0.6)  // softer so they don't dominate
      }
    }

    // Effect-specific particle init
    if (activeEffect === 'embers') {
      for (const p of particles) {
        p.vy = -(Math.random() * 0.8 + 0.2)
        p.vx = (Math.random() - 0.5) * 0.6
        p.size = Math.random() * 3 + 1
      }
    } else if (activeEffect === 'snow') {
      for (const p of particles) {
        p.size = Math.random() * 5 + 2  // bigger for snowflake detail
        p.vy = Math.random() * 0.6 + 0.15  // slow gentle fall
        p.vx = (Math.random() - 0.5) * 0.3  // slight drift
        p.vr = (Math.random() - 0.5) * 0.008  // slow rotation
      }
    } else if (activeEffect === 'sakura') {
      for (const p of particles) {
        p.size = Math.random() * 3 + 2
        p.vy = Math.random() * 0.5 + 0.2    // gentle fall
        p.vx = Math.random() * 0.3 + 0.1    // slight lateral drift
        p.vr = (Math.random() - 0.5) * 0.015 // slow tumble
      }
    } else if (activeEffect === 'flames') {
      // Flame tongues — spread across bottom, staggered heights
      for (const p of particles) {
        p.x = Math.random() * canvas.width
        p.y = canvas.height - Math.random() * canvas.height * 0.08  // near bottom
        p.vy = -(Math.random() * 0.4 + 0.15)  // slow rise
        p.size = Math.random() * 5 + 2
        p.opacity = Math.random() * 0.4 + 0.4
      }
    }

    let t = 0
    const animate = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height)
      t++

      const w = canvas.width
      const h = canvas.height

      if (activeEffect === 'digital-rain') {
        drawDigitalRain(ctx, rainColumns, h, t)
        rafRef.current = requestAnimationFrame(animate)
        return
      }

      // 3D starfield has its own complete render path
      if (activeEffect === 'stars') {
        const speed = foreground ? STAR_SPEED_FG : STAR_SPEED
        drawStarfield3D(ctx, stars3D, w / 2, h / 2, w, h, speed)
        rafRef.current = requestAnimationFrame(animate)
        return
      }

      // Flames use a batch draw (base glow + tongues together)
      if (activeEffect === 'flames') {
        drawFlames(ctx, particles, t, w, h)
        // Move flame particles
        for (const p of particles) {
          p.y += p.vy
          if (p.y < h * 0.3) {
            // Reset when risen too high
            p.y = h - Math.random() * h * 0.05
            p.x = Math.random() * w
            p.vy = -(Math.random() * 0.4 + 0.15)
            p.opacity = Math.random() * 0.4 + 0.4
            p.phase = Math.random() * Math.PI * 2
            p.size = Math.random() * 5 + 2
          }
        }
        rafRef.current = requestAnimationFrame(animate)
        return
      }

      for (const p of particles) {
        // Draw
        switch (activeEffect) {
          case 'snow':      drawSnowflake(ctx, p); break
          case 'leaves':    drawLeaf(ctx, p); break
          case 'rain':      drawRain(ctx, p, w); break
          case 'fireflies': drawFirefly(ctx, p, t); break
          // stars handled above via 3D system
          case 'sakura':    drawSakuraPetal(ctx, p); break
          case 'embers':    drawEmber(ctx, p, t); break
        }

        // Move
        if (activeEffect === 'fireflies') {
          p.x += Math.sin((p.phase ?? 0) + t * 0.01) * 0.5
          p.y += Math.sin((p.phase ?? 0) * 1.3 + t * 0.008) * 0.4
        } else if (activeEffect === 'rain') {
          p.x += 1.5
          p.y += 12
        } else if (activeEffect === 'embers') {
          p.x += p.vx + Math.sin((p.phase ?? 0) + t * 0.015) * 0.3
          p.y += p.vy
          p.opacity -= 0.001
          if (p.opacity < 0.05) {
            p.opacity = Math.random() * 0.7 + 0.3
            p.y = h + 10
            p.x = Math.random() * w
          }
        } else {
          p.x += p.vx
          p.y += p.vy
          if (p.rotation !== undefined && p.vr !== undefined) p.rotation += p.vr
        }

        // Wrap (embers handle their own)
        if (activeEffect !== 'embers') {
          if (p.y > h + 20) p.y = -20
          if (p.y < -20)    p.y = h + 20
          if (p.x > w + 20) p.x = -20
          if (p.x < -20)    p.x = w + 20
        }
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
        zIndex: foreground ? 10 : 0,
      }}
    />
  )
}
