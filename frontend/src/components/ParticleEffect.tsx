import { useEffect, useRef } from 'react'
import { CSSParticleEffect } from './CSSParticleEffect'

export type EffectName = 'snow' | 'leaves' | 'rain' | 'fireflies' | 'stars' | 'sakura' | 'embers' | 'digital-rain' | 'flames' | 'water' | 'boba' | 'clouds' | 'fruit' | 'junkfood' | 'warzone' | 'hearts' | 'cactus' | 'candy' | 'wasteland' | 'fireworks' | 'forest' | 'none'

// Effects that use GPU-composited CSS animations instead of canvas
const CSS_EFFECTS = new Set<EffectName>(['leaves', 'snow', 'fruit', 'junkfood', 'sakura', 'hearts', 'cactus', 'candy'])

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

// (Snowflake, leaf draw functions moved to CSSParticleEffect — GPU-composited)

// ── Rain + Lightning ──

function drawRain(ctx: CanvasRenderingContext2D, p: Particle, w: number) {
  ctx.beginPath()
  ctx.moveTo(p.x, p.y)
  ctx.lineTo(p.x + w * 0.01, p.y + p.size * 4)
  ctx.strokeStyle = `rgba(150, 190, 230, ${p.opacity * 0.6})`
  ctx.lineWidth = 1
  ctx.stroke()
}

interface LightningState {
  active: boolean
  x: number           // bolt origin x
  opacity: number     // current flash brightness (decays)
  segments: { x: number; y: number }[]  // bolt path
  startFrame: number
  duration: number    // frames the flash lasts
}

function triggerLightning(w: number, h: number, t: number): LightningState {
  const x = w * 0.15 + Math.random() * w * 0.7
  // Build a jagged bolt from top to ~70% down
  const segments: { x: number; y: number }[] = [{ x, y: 0 }]
  let cx = x
  let cy = 0
  const endY = h * (0.5 + Math.random() * 0.3)
  const stepCount = 8 + Math.floor(Math.random() * 6)
  const stepY = endY / stepCount
  for (let i = 0; i < stepCount; i++) {
    cx += (Math.random() - 0.5) * 80
    cy += stepY + (Math.random() - 0.5) * stepY * 0.3
    segments.push({ x: cx, y: cy })
  }
  return { active: true, x, opacity: 1, segments, startFrame: t, duration: 8 + Math.floor(Math.random() * 8) }
}

function drawLightning(ctx: CanvasRenderingContext2D, ls: LightningState, w: number, h: number, t: number) {
  const elapsed = t - ls.startFrame
  if (elapsed > ls.duration) { ls.active = false; return }

  // Rapid flash then decay — 2 sharp pulses then fade
  let alpha: number
  if (elapsed < 2) alpha = 1.0
  else if (elapsed < 4) alpha = 0.3
  else if (elapsed < 5) alpha = 0.7  // secondary flash
  else alpha = Math.max(0, 1 - (elapsed - 5) / (ls.duration - 5))

  ls.opacity = alpha

  // Screen flash (full canvas white overlay)
  ctx.fillStyle = `rgba(200, 210, 240, ${alpha * 0.08})`
  ctx.fillRect(0, 0, w, h)

  // Draw bolt
  if (ls.segments.length > 1) {
    // Glow layer
    ctx.strokeStyle = `rgba(180, 200, 255, ${alpha * 0.3})`
    ctx.lineWidth = 6
    ctx.beginPath()
    ctx.moveTo(ls.segments[0].x, ls.segments[0].y)
    for (let i = 1; i < ls.segments.length; i++) {
      ctx.lineTo(ls.segments[i].x, ls.segments[i].y)
    }
    ctx.stroke()

    // Core bolt
    ctx.strokeStyle = `rgba(230, 240, 255, ${alpha * 0.8})`
    ctx.lineWidth = 2
    ctx.beginPath()
    ctx.moveTo(ls.segments[0].x, ls.segments[0].y)
    for (let i = 1; i < ls.segments.length; i++) {
      ctx.lineTo(ls.segments[i].x, ls.segments[i].y)
    }
    ctx.stroke()

    // Bright center
    ctx.strokeStyle = `rgba(255, 255, 255, ${alpha * 0.9})`
    ctx.lineWidth = 1
    ctx.beginPath()
    ctx.moveTo(ls.segments[0].x, ls.segments[0].y)
    for (let i = 1; i < ls.segments.length; i++) {
      ctx.lineTo(ls.segments[i].x, ls.segments[i].y)
    }
    ctx.stroke()
  }
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

// (Sakura draw function moved to CSSParticleEffect — GPU-composited)

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

// ── Flames: additive-blend particles with aging color (white→yellow→orange→red) ──

interface FlameParticle {
  x: number
  y: number
  vx: number
  vy: number
  life: number     // current age (0 = just born)
  maxLife: number   // total lifespan
  size: number
}

function initFlameParticles(count: number, w: number, h: number): FlameParticle[] {
  return Array.from({ length: count }, () => spawnFlame(w, h))
}

function spawnFlame(w: number, h: number): FlameParticle {
  return {
    x: Math.random() * w,
    y: h + Math.random() * 10,    // spawn at/below bottom edge
    vx: (Math.random() - 0.5) * 1.5,
    vy: -(Math.random() * 3 + 1.5), // rise upward
    life: 0,
    maxLife: 40 + Math.random() * 40,
    size: 15 + Math.random() * 25,
  }
}

function drawFlames(ctx: CanvasRenderingContext2D, flames: FlameParticle[], w: number, h: number) {
  // Semi-transparent black clear for natural fade trails
  ctx.globalCompositeOperation = 'source-over'
  ctx.fillStyle = 'rgba(0, 0, 0, 0.12)'
  ctx.fillRect(0, 0, w, h)

  // Additive blending: overlapping particles add colors → white-hot center
  ctx.globalCompositeOperation = 'lighter'

  for (const f of flames) {
    const lifeRatio = f.life / f.maxLife  // 0 = newborn, 1 = dying

    // Skip dead particles
    if (lifeRatio >= 1) continue

    // Size shrinks as particle ages
    const radius = f.size * (1 - lifeRatio * 0.6)

    // Opacity fades out
    const alpha = (1 - lifeRatio) * 0.35

    // Color shifts with age: white-yellow → orange → red → dark red
    let r: number, g: number, b: number
    if (lifeRatio < 0.2) {
      // Young: bright white-yellow
      r = 255
      g = 255 - lifeRatio * 200
      b = 200 - lifeRatio * 1000
    } else if (lifeRatio < 0.5) {
      // Mid: orange
      const t = (lifeRatio - 0.2) / 0.3
      r = 255
      g = Math.floor(215 - t * 150)
      b = 0
    } else {
      // Old: red fading to dark
      const t = (lifeRatio - 0.5) / 0.5
      r = Math.floor(255 - t * 155)
      g = Math.floor(65 - t * 65)
      b = 0
    }

    // Radial gradient for soft glow
    const gradient = ctx.createRadialGradient(f.x, f.y, 0, f.x, f.y, radius)
    gradient.addColorStop(0, `rgba(${r}, ${g}, ${b}, ${alpha})`)
    gradient.addColorStop(0.4, `rgba(${r}, ${Math.floor(g * 0.7)}, ${b}, ${alpha * 0.6})`)
    gradient.addColorStop(1, `rgba(${Math.floor(r * 0.5)}, 0, 0, 0)`)

    ctx.beginPath()
    ctx.arc(f.x, f.y, radius, 0, Math.PI * 2)
    ctx.fillStyle = gradient
    ctx.fill()
  }

  // Reset composite operation
  ctx.globalCompositeOperation = 'source-over'
}

function updateFlames(flames: FlameParticle[], w: number, h: number) {
  for (let i = 0; i < flames.length; i++) {
    const f = flames[i]
    // Move
    f.x += f.vx + Math.sin(f.life * 0.08) * 0.8  // wobble
    f.y += f.vy
    f.vy *= 0.99  // slow down slightly as it rises
    f.life++

    // Respawn when dead
    if (f.life >= f.maxLife || f.y < -50) {
      flames[i] = spawnFlame(w, h)
    }
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
  opacity: number   // base opacity for this column
}

function initRainColumns(w: number, h: number, layer: 'bg' | 'fg' = 'bg'): RainColumn[] {
  if (layer === 'bg') {
    // Background: dense grid with varied depth — size drives speed & opacity
    // depth 0 = far (small, slow, dim), depth 1 = near (larger, faster, bright)
    const spacing = 11                                    // tight spacing for full coverage
    const cols = Math.ceil(w / spacing) + 1
    return Array.from({ length: cols }, (_, i) => {
      const depth = Math.random()                         // 0=far, 1=near
      const charSize = Math.round(10 + depth * 8)         // 10–18px
      const baseSpeed = 0.6 + depth * 2.4                 // 0.6–3.0 base
      const speed = baseSpeed + (Math.random() - 0.5) * 1.2  // ±0.6 randomness
      const opacity = 0.3 + depth * 0.6                   // 0.3–0.9
      return {
        x: i * spacing,
        chars: Array.from({ length: Math.floor(h / charSize) + 10 }, () =>
          MATRIX_CHARS[Math.floor(Math.random() * MATRIX_CHARS.length)]
        ),
        y: Math.random() * h * 2 - h,
        speed: Math.max(0.3, speed),
        length: Math.floor(Math.random() * 12) + 8,
        charSize,
        opacity,
      }
    })
  }

  // Foreground: sparse, larger columns at random positions
  const fgColumns: RainColumn[] = []

  // Regular foreground columns: ~8-12, larger (22-28px), bright
  const fgCount = 8 + Math.floor(Math.random() * 5)
  for (let i = 0; i < fgCount; i++) {
    const charSize = 22 + Math.random() * 6
    const baseSpeed = 3.5 + Math.random() * 2.5           // 3.5–6.0
    fgColumns.push({
      x: Math.random() * w,
      chars: Array.from({ length: Math.floor(h / charSize) + 10 }, () =>
        MATRIX_CHARS[Math.floor(Math.random() * MATRIX_CHARS.length)]
      ),
      y: Math.random() * h * 2 - h,
      speed: baseSpeed,
      length: Math.floor(Math.random() * 8) + 5,
      charSize,
      opacity: 0.9,
    })
  }

  // Extreme close-up columns: very rare — 85% chance of 0, 15% chance of 1
  const extremeCount = Math.random() < 0.15 ? 1 : 0
  for (let i = 0; i < extremeCount; i++) {
    const charSize = 48 + Math.random() * 24
    fgColumns.push({
      x: Math.random() * w,
      chars: Array.from({ length: Math.floor(h / charSize) + 10 }, () =>
        MATRIX_CHARS[Math.floor(Math.random() * MATRIX_CHARS.length)]
      ),
      y: Math.random() * h * 2 - h,
      speed: 5 + Math.random() * 4,                       // 5–9, fastest
      length: Math.floor(Math.random() * 5) + 3,
      charSize,
      opacity: 0.95,                                       // almost opaque — right in your face
    })
  }

  return fgColumns
}

function drawDigitalRain(ctx: CanvasRenderingContext2D, columns: RainColumn[], w: number, h: number, _t: number) {
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
        ctx.fillStyle = `rgba(180, 255, 180, ${col.opacity})`
        ctx.font = `bold ${col.charSize}px "Courier New", monospace`
      } else {
        const fade = 1 - (i / col.length)
        const g = Math.floor(180 * fade + 40)
        ctx.fillStyle = `rgba(0, ${g}, 0, ${fade * col.opacity})`
        ctx.font = `${col.charSize}px "Courier New", monospace`
      }
      ctx.fillText(ch, col.x, cy)
    }

    col.y += col.speed
    if (col.y - col.length * col.charSize > h) {
      // Respawn with speed proportional to size (closer = faster) + randomness
      if (col.charSize > 40) {
        // Extreme: very long off-screen delay before next appearance
        col.y = -(h * (8 + Math.random() * 15))       // 8–23 screen-heights above
        col.x = Math.random() * w                     // new random x position
        col.speed = 5 + Math.random() * 4             // extreme: 5–9
      } else if (col.charSize > 20) {
        col.y = -col.length * col.charSize * Math.random()
        col.speed = 3.5 + Math.random() * 2.5         // foreground: 3.5–6
      } else {
        // Background: depth-based — larger chars faster
        const depth = (col.charSize - 10) / 8          // 0–1 from charSize 10–18
        const base = 0.6 + depth * 2.4
        col.speed = Math.max(0.3, base + (Math.random() - 0.5) * 1.2)
        col.y = -col.length * col.charSize * Math.random()
      }
      col.length = col.charSize > 40
        ? Math.floor(Math.random() * 5) + 3
        : col.charSize > 20
          ? Math.floor(Math.random() * 8) + 5
          : Math.floor(Math.random() * 12) + 8
    }
  }
}

// ── Water caustics (Voronoi-based) + bubbles ──

interface WaterPoint {
  x: number; y: number; vx: number; vy: number
}

interface Bubble {
  x: number; y: number; r: number; speed: number; wobblePhase: number; opacity: number
}

interface WaterState {
  points: WaterPoint[]
  offscreen: HTMLCanvasElement
  offCtx: CanvasRenderingContext2D
  scale: number
  bubbles: Bubble[]
}

function initWaterState(w: number, h: number, foreground: boolean): WaterState | null {
  const scale = 6  // render at 1/6 resolution for performance
  const ow = Math.ceil(w / scale)
  const oh = Math.ceil(h / scale)
  const offscreen = document.createElement('canvas')
  offscreen.width = ow
  offscreen.height = oh
  const offCtx = offscreen.getContext('2d')
  if (!offCtx) return null

  // Wandering Voronoi seed points
  const pointCount = foreground ? 0 : 14
  const points: WaterPoint[] = Array.from({ length: pointCount }, () => ({
    x: Math.random() * ow,
    y: Math.random() * oh,
    vx: (Math.random() - 0.5) * 0.4,
    vy: (Math.random() - 0.5) * 0.4,
  }))

  const bubbleCount = foreground ? 12 : 60
  const bubbles: Bubble[] = Array.from({ length: bubbleCount }, () => ({
    x: Math.random() * w,
    y: h + Math.random() * h,
    r: foreground ? 4 + Math.random() * 8 : 2 + Math.random() * 5,
    speed: 0.4 + Math.random() * 1.0,
    wobblePhase: Math.random() * Math.PI * 2,
    opacity: foreground ? 0.4 + Math.random() * 0.4 : 0.2 + Math.random() * 0.4,
  }))

  return { points, offscreen, offCtx, scale, bubbles }
}

function drawWater(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  t: number,
  state: WaterState,
  foreground: boolean,
) {
  // ── Voronoi caustic pattern (background only) ──
  if (!foreground && state.points.length > 0) {
    const { points, offscreen, offCtx, scale } = state
    const ow = offscreen.width
    const oh = offscreen.height

    // Move seed points (gentle drift with wrap)
    for (const p of points) {
      p.x += p.vx + Math.sin(t * 0.001 + p.y * 0.1) * 0.15
      p.y += p.vy + Math.cos(t * 0.0008 + p.x * 0.1) * 0.15
      if (p.x < 0) p.x += ow; if (p.x >= ow) p.x -= ow
      if (p.y < 0) p.y += oh; if (p.y >= oh) p.y -= oh
    }

    // Render caustics at low res: for each pixel, find distance to two nearest points
    const imgData = offCtx.createImageData(ow, oh)
    const data = imgData.data
    for (let py = 0; py < oh; py++) {
      for (let px = 0; px < ow; px++) {
        let d1 = 1e9, d2 = 1e9
        for (const pt of points) {
          // Toroidal distance for seamless wrapping
          let dx = Math.abs(px - pt.x)
          let dy = Math.abs(py - pt.y)
          if (dx > ow / 2) dx = ow - dx
          if (dy > oh / 2) dy = oh - dy
          const d = dx * dx + dy * dy
          if (d < d1) { d2 = d1; d1 = d }
          else if (d < d2) { d2 = d }
        }
        // Caustic brightness: bright where d2 - d1 is small (cell edges)
        const edge = Math.sqrt(d2) - Math.sqrt(d1)
        const brightness = Math.max(0, 1 - edge / 8)  // sharp bright edges
        const caustic = brightness * brightness * 255   // quadratic falloff

        const idx = (py * ow + px) * 4
        // Aqua-tinted light: brighter = whiter, dimmer = blue
        data[idx    ] = Math.min(255, caustic * 0.7 + 20)  // R
        data[idx + 1] = Math.min(255, caustic * 0.9 + 40)  // G
        data[idx + 2] = Math.min(255, caustic + 60)         // B
        data[idx + 3] = Math.min(255, caustic * 0.8)        // A
      }
    }
    offCtx.putImageData(imgData, 0, 0)

    // Draw upscaled to main canvas with additive blending
    ctx.save()
    ctx.globalCompositeOperation = 'lighter'
    ctx.imageSmoothingEnabled = true
    ctx.drawImage(offscreen, 0, 0, w, h)
    ctx.restore()
  }

  // ── Bubbles ──
  for (const b of state.bubbles) {
    const wobble = Math.sin(t * 0.002 + b.wobblePhase) * 2.5
    const bx = b.x + wobble

    ctx.save()

    // Glow
    const g = ctx.createRadialGradient(bx, b.y, b.r * 0.1, bx, b.y, b.r)
    g.addColorStop(0, `rgba(220, 245, 255, ${b.opacity * 0.5})`)
    g.addColorStop(0.6, `rgba(160, 220, 255, ${b.opacity * 0.2})`)
    g.addColorStop(1, 'rgba(160, 220, 255, 0)')
    ctx.fillStyle = g
    ctx.beginPath()
    ctx.arc(bx, b.y, b.r, 0, Math.PI * 2)
    ctx.fill()

    // Rim
    ctx.strokeStyle = `rgba(230, 248, 255, ${b.opacity * 0.6})`
    ctx.lineWidth = 0.7
    ctx.stroke()

    // Specular
    ctx.fillStyle = `rgba(255, 255, 255, ${b.opacity * 0.8})`
    ctx.beginPath()
    ctx.arc(bx - b.r * 0.3, b.y - b.r * 0.3, b.r * 0.22, 0, Math.PI * 2)
    ctx.fill()

    ctx.restore()

    // Animate
    b.y -= b.speed
    b.x += wobble * 0.015
    if (b.y < -b.r * 2) {
      b.y = h + b.r * 2 + Math.random() * 60
      b.x = Math.random() * w
    }
  }
}

// ── Clouds (layered fluffy blobs drifting across the sky) ──

interface CloudBlob {
  x: number; y: number; rx: number; ry: number  // ellipse radii
  baseAlpha: number
}

interface CloudGroup {
  blobs: CloudBlob[]   // cluster of overlapping ellipses = one cloud
  x: number            // group offset
  y: number
  speed: number        // drift speed (px/frame)
  scale: number        // depth scale (far=small, near=big)
  alpha: number        // depth alpha
}

interface CloudState {
  groups: CloudGroup[]
  offscreen: HTMLCanvasElement
  offCtx: CanvasRenderingContext2D
}

function makeCloudGroup(w: number, h: number, layer: 'far' | 'mid' | 'near'): CloudGroup {
  const configs = {
    far:  { scale: 0.7,  alpha: 0.5,  speed: 0.1 + Math.random() * 0.1, yMin: 0.02, yMax: 0.92, blobCount: 5 },
    mid:  { scale: 1.0,  alpha: 0.7,  speed: 0.25 + Math.random() * 0.15, yMin: 0.05, yMax: 0.88, blobCount: 6 },
    near: { scale: 1.5,  alpha: 0.85, speed: 0.5 + Math.random() * 0.3, yMin: 0.05, yMax: 0.85, blobCount: 7 },
  }
  const c = configs[layer]
  const cx = Math.random() * w * 1.5 - w * 0.25
  const cy = h * c.yMin + Math.random() * h * (c.yMax - c.yMin)

  // Generate cluster of overlapping ellipses
  const blobs: CloudBlob[] = Array.from({ length: c.blobCount + Math.floor(Math.random() * 3) }, () => ({
    x: (Math.random() - 0.5) * 120 * c.scale,
    y: (Math.random() - 0.5) * 40 * c.scale,
    rx: (40 + Math.random() * 60) * c.scale,
    ry: (25 + Math.random() * 25) * c.scale,
    baseAlpha: 0.5 + Math.random() * 0.5,
  }))

  return { blobs, x: cx, y: cy, speed: c.speed, scale: c.scale, alpha: c.alpha }
}

function initCloudState(w: number, h: number, foreground: boolean): CloudState | null {
  const offscreen = document.createElement('canvas')
  offscreen.width = w
  offscreen.height = h
  const offCtx = offscreen.getContext('2d')
  if (!offCtx) return null

  const groups: CloudGroup[] = []
  if (!foreground) {
    // Spread clouds across the full screen with staggered start positions
    // Far layer: small, slow, faint
    for (let i = 0; i < 14; i++) groups.push(makeCloudGroup(w, h, 'far'))
    // Mid layer
    for (let i = 0; i < 10; i++) groups.push(makeCloudGroup(w, h, 'mid'))
    // Near layer: big, fast, more opaque
    for (let i = 0; i < 6; i++) groups.push(makeCloudGroup(w, h, 'near'))
  } else {
    // Foreground: 1-2 very large close clouds
    for (let i = 0; i < 2; i++) {
      const g = makeCloudGroup(w, h, 'near')
      g.scale = 2.0
      g.alpha = 0.3
      g.speed = 0.7 + Math.random() * 0.4
      g.y = h * 0.05 + Math.random() * h * 0.5
      for (const b of g.blobs) { b.rx *= 1.8; b.ry *= 1.8; b.x *= 1.8; b.y *= 1.8 }
      groups.push(g)
    }
  }

  return { groups, offscreen, offCtx }
}

function drawClouds(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  _t: number,
  state: CloudState,
) {
  const { groups, offscreen, offCtx } = state
  offCtx.clearRect(0, 0, w, h)

  for (const g of groups) {
    // Draw each blob in the group
    for (const b of g.blobs) {
      const bx = g.x + b.x
      const by = g.y + b.y

      const grad = offCtx.createRadialGradient(bx, by, 0, bx, by, Math.max(b.rx, b.ry))
      const a = g.alpha * b.baseAlpha
      grad.addColorStop(0, `rgba(255, 255, 255, ${a})`)
      grad.addColorStop(0.4, `rgba(255, 255, 255, ${a * 0.8})`)
      grad.addColorStop(0.7, `rgba(245, 248, 255, ${a * 0.4})`)
      grad.addColorStop(1, 'rgba(245, 248, 255, 0)')

      offCtx.save()
      offCtx.translate(bx, by)
      offCtx.scale(1, b.ry / b.rx)  // squash to ellipse
      offCtx.beginPath()
      offCtx.arc(0, 0, b.rx, 0, Math.PI * 2)
      offCtx.fillStyle = grad
      offCtx.fill()
      offCtx.restore()
    }

    // Drift
    g.x += g.speed
    // Wrap around when fully off-screen right — reappear from left at new Y
    const maxBlobR = Math.max(...g.blobs.map(b => b.rx + Math.abs(b.x))) * 1.5
    if (g.x - maxBlobR > w) {
      g.x = -maxBlobR * 2
      g.y = h * 0.02 + Math.random() * h * 0.90  // full screen height coverage
    }
  }

  // Composite onto main canvas
  ctx.drawImage(offscreen, 0, 0)
}

// ── Warzone: Terminator apocalyptic rubble wasteland + lasers ──
interface RubbleMound {
  points: { x: number; y: number }[]  // organic curve control points
  shade: number  // 0-1, darker for back layers
}
interface LaserShot {
  fromLeft: boolean; y: number; angle: number
  color: string; width: number; life: number; maxLife: number
}
interface WarzoneState {
  mounds: RubbleMound[]
  groundY: number
  laserCooldown: number
  lasers: LaserShot[]
  burstRemaining: number
  burstY: number; burstFromLeft: boolean; burstColor: string
  burstAngle: number; burstWidth: number; burstDelay: number; burstDelayTimer: number
  flashAlpha: number; flashTimer: number
}

const LASER_COLORS = ['rgba(255,20,40,A)', 'rgba(60,140,255,A)', 'rgba(180,60,255,A)', 'rgba(255,100,20,A)']

function initWarzoneState(w: number, h: number, foreground: boolean): WarzoneState {
  const groundY = h * 0.5  // rubble starts halfway — fills bottom half
  const mounds: RubbleMound[] = []
  const empty: WarzoneState = {
    mounds, groundY, laserCooldown: 90, lasers: [], burstRemaining: 0,
    burstY: 0, burstFromLeft: true, burstColor: '', burstAngle: 0,
    burstWidth: 1.5, burstDelay: 3, burstDelayTimer: 0, flashAlpha: 0, flashTimer: 200,
  }
  if (foreground) return empty

  // 3 overlapping layers of organic rubble mounds — back to front
  for (let layer = 0; layer < 3; layer++) {
    const baseY = groundY + layer * h * 0.1
    const moundCount = 6 + Math.floor(Math.random() * 4)
    for (let i = 0; i < moundCount; i++) {
      const cx = (w / moundCount) * i + (Math.random() - 0.5) * w * 0.2
      const mw = 80 + Math.random() * 160
      const mh = 30 + Math.random() * 70 + (2 - layer) * 25  // back layers taller
      const ptCount = 7 + Math.floor(Math.random() * 5)
      const points: { x: number; y: number }[] = []
      for (let p = 0; p <= ptCount; p++) {
        const frac = p / ptCount
        const px = cx - mw / 2 + frac * mw
        const mainHump = Math.sin(frac * Math.PI) * mh
        const bump1 = Math.sin(frac * Math.PI * 2.7 + i) * mh * 0.25
        const bump2 = Math.sin(frac * Math.PI * 5.3 + layer * 2) * mh * 0.12
        const jitter = (Math.random() - 0.5) * mh * 0.18
        const py = baseY - mainHump - bump1 - bump2 + jitter
        points.push({ x: px, y: Math.min(py, baseY + 5) })
      }
      points[0].y = baseY + 15
      points[points.length - 1].y = baseY + 15
      mounds.push({ points, shade: 0.3 + layer * 0.25 })  // back=darker, front=lighter
    }
  }
  // Sort so back (darker/taller) layers draw first
  mounds.sort((a, b) => a.shade - b.shade)

  return { ...empty, mounds, groundY }
}

function drawWarzone(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  t: number,
  state: WarzoneState,
  foreground: boolean,
) {
  if (foreground) return
  ctx.save()
  const gY = state.groundY

  // ── Fiery red/orange glow behind rubble (horizon fire) ──
  const glow1 = ctx.createRadialGradient(w * 0.5, gY + 20, 0, w * 0.5, gY + 20, w * 0.7)
  glow1.addColorStop(0, 'rgba(200,50,15,0.30)')
  glow1.addColorStop(0.25, 'rgba(180,35,10,0.18)')
  glow1.addColorStop(0.6, 'rgba(120,20,5,0.08)')
  glow1.addColorStop(1, 'rgba(40,5,0,0)')
  ctx.fillStyle = glow1
  ctx.fillRect(0, 0, w, h)

  const glow2 = ctx.createRadialGradient(w * 0.25, gY + 30, 0, w * 0.25, gY + 30, w * 0.45)
  glow2.addColorStop(0, 'rgba(220,80,15,0.15)')
  glow2.addColorStop(0.5, 'rgba(160,40,8,0.07)')
  glow2.addColorStop(1, 'rgba(60,10,0,0)')
  ctx.fillStyle = glow2
  ctx.fillRect(0, 0, w, h)

  const glow3 = ctx.createRadialGradient(w * 0.75, gY + 10, 0, w * 0.75, gY + 10, w * 0.35)
  glow3.addColorStop(0, 'rgba(200,60,20,0.12)')
  glow3.addColorStop(1, 'rgba(80,15,0,0)')
  ctx.fillStyle = glow3
  ctx.fillRect(0, 0, w, h)

  // ── Rubble mounds — organic curves, back to front ──
  for (const mound of state.mounds) {
    const shade = Math.floor(8 + mound.shade * 8)
    ctx.fillStyle = `rgb(${shade},${shade + 1},${shade + 3})`
    ctx.beginPath()
    const pts = mound.points
    ctx.moveTo(pts[0].x, pts[0].y)
    // Smooth curve through points
    for (let i = 1; i < pts.length - 1; i++) {
      const xc = (pts[i].x + pts[i + 1].x) / 2
      const yc = (pts[i].y + pts[i + 1].y) / 2
      ctx.quadraticCurveTo(pts[i].x, pts[i].y, xc, yc)
    }
    ctx.lineTo(pts[pts.length - 1].x, pts[pts.length - 1].y)
    // Close along bottom
    ctx.lineTo(pts[pts.length - 1].x, h + 10)
    ctx.lineTo(pts[0].x, h + 10)
    ctx.closePath()
    ctx.fill()
  }

  // ── Laser burst system ──
  // Manage active burst
  if (state.burstRemaining > 0) {
    state.burstDelayTimer--
    if (state.burstDelayTimer <= 0) {
      const yJitter = (Math.random() - 0.5) * 8
      const shot: LaserShot = {
        fromLeft: state.burstFromLeft,
        y: state.burstY + yJitter,
        angle: state.burstAngle + (Math.random() - 0.5) * 0.02,
        color: state.burstColor,
        width: state.burstWidth,
        life: 8 + Math.floor(Math.random() * 12),
        maxLife: 0,
      }
      shot.maxLife = shot.life
      state.lasers.push(shot)
      state.burstRemaining--
      state.burstDelayTimer = state.burstDelay
    }
  }

  // Spawn new burst
  state.laserCooldown--
  if (state.laserCooldown <= 0 && state.burstRemaining <= 0) {
    state.burstRemaining = 3 + Math.floor(Math.random() * 8)
    state.burstFromLeft = Math.random() > 0.5
    state.burstColor = LASER_COLORS[Math.floor(Math.random() * LASER_COLORS.length)]
    state.burstY = h * 0.1 + Math.random() * h * 0.35  // above the rubble line
    state.burstAngle = (Math.random() - 0.5) * 0.12
    state.burstWidth = 1.5 + Math.random() * 1.5
    state.burstDelay = 2 + Math.floor(Math.random() * 3)
    state.burstDelayTimer = 0
    state.laserCooldown = 90 + Math.floor(Math.random() * 200)
  }

  // Draw lasers
  ctx.globalCompositeOperation = 'lighter'
  for (let i = state.lasers.length - 1; i >= 0; i--) {
    const l = state.lasers[i]
    l.life--
    if (l.life <= 0) { state.lasers.splice(i, 1); continue }
    const fade = l.life / l.maxLife
    const env = fade > 0.7 ? 1.0 : fade / 0.7
    const a = 0.8 * env
    const sx = l.fromLeft ? -10 : w + 10
    const ex = l.fromLeft ? w + 10 : -10
    const sy = l.y - Math.tan(l.angle) * (l.fromLeft ? 0 : w)
    const ey = l.y + Math.tan(l.angle) * (l.fromLeft ? w : 0)
    ctx.strokeStyle = l.color.replace('A', String(a))
    ctx.lineWidth = l.width
    ctx.beginPath(); ctx.moveTo(sx, sy); ctx.lineTo(ex, ey); ctx.stroke()
    ctx.strokeStyle = l.color.replace('A', String(a * 0.2))
    ctx.lineWidth = l.width * 5
    ctx.stroke()
  }
  ctx.globalCompositeOperation = 'source-over'

  // ── Explosion flashes ──
  state.flashTimer--
  if (state.flashTimer <= 0) {
    state.flashAlpha = 0.12 + Math.random() * 0.1
    state.flashTimer = 200 + Math.floor(Math.random() * 400)
  }
  if (state.flashAlpha > 0) {
    const colors = ['rgba(255,130,35,A)', 'rgba(255,50,25,A)', 'rgba(200,170,255,A)']
    ctx.fillStyle = colors[Math.floor(Math.random() * colors.length)].replace('A', String(state.flashAlpha))
    ctx.fillRect(0, 0, w, h)
    state.flashAlpha *= 0.9
    if (state.flashAlpha < 0.005) state.flashAlpha = 0
  }

  ctx.restore()
}

// ── Winter landscape (static background for snow theme) ──

interface ChristmasLight {
  x: number; y: number; color: string; phase: number; radius: number
}

interface WinterTree {
  x: number; groundY: number; scale: number; layer: number
  trunkH: number; treeH: number; baseW: number; tiers: number
  snowCoverage: number   // 0-1: how much snow on tips (0 = bare, 1 = full caps)
  greenHue: number       // variation in green shade
  lean: number           // slight tilt (-0.1 to 0.1)
  lights: ChristmasLight[]
}
interface WinterState {
  trees: WinterTree[][]  // per layer
}

function initWinterState(w: number, h: number): WinterState {
  const hillBase = [h * 0.55, h * 0.65, h * 0.78]
  const hillAmp = [h * 0.08, h * 0.1, h * 0.07]
  const hillFreq = [0.003, 0.005, 0.004]
  const hillPhase = [0, 1.5, 3.2]
  const trees: WinterTree[][] = []

  for (let layer = 0; layer < 3; layer++) {
    const layerTrees: WinterTree[] = []
    const treeCount = layer === 0 ? 8 : layer === 1 ? 12 : 16
    const treeScale = layer === 0 ? 0.5 : layer === 1 ? 0.7 : 1.0
    for (let t = 0; t < treeCount; t++) {
      // Distribute evenly across full width with moderate random jitter
      const spacing = w / treeCount
      const jitter = (Math.random() - 0.5) * spacing * 0.6
      const tx = spacing * (t + 0.5) + jitter
      const hillY = hillBase[layer]
        - Math.sin(tx * hillFreq[layer] + hillPhase[layer]) * hillAmp[layer]
        - Math.sin(tx * hillFreq[layer] * 2.3 + hillPhase[layer] * 1.7) * hillAmp[layer] * 0.3
      const trunkH = (6 + Math.random() * 6) * treeScale
      const treeH = (20 + Math.random() * 35) * treeScale  // more height variation
      const baseW = (12 + Math.random() * 14) * treeScale  // more width variation
      const tiers = 2 + Math.floor(Math.random() * 3)      // 2-4 tiers

      // Pre-generate Christmas light positions along tree edges
      const lightColors = ['#ff2020', '#00cc44', '#ffcc00', '#2288ff', '#ff6600', '#ff44aa']
      const lights: ChristmasLight[] = []
      for (let i = 0; i < tiers; i++) {
        const frac = i / tiers
        const tierY = hillY - trunkH - frac * treeH
        const tierW = baseW * (1 - frac * 0.6)
        const tierH = treeH / tiers * 1.4
        // Place lights along the left and right edges of each tier
        const lightsPerSide = Math.max(2, Math.floor(3 * treeScale))
        for (let s = 0; s < lightsPerSide; s++) {
          const edgeFrac = (s + 0.5) / lightsPerSide  // 0..1 along edge
          const ey = tierY - tierH * (1 - edgeFrac)   // from top to bottom
          const edgeW = tierW * edgeFrac               // width at this height
          // Left edge light
          lights.push({
            x: tx - edgeW + (Math.random() - 0.5) * 2 * treeScale,
            y: ey + (Math.random() - 0.5) * 2 * treeScale,
            color: lightColors[Math.floor(Math.random() * lightColors.length)],
            phase: Math.random() * Math.PI * 2,
            radius: (1.5 + Math.random()) * treeScale,
          })
          // Right edge light
          lights.push({
            x: tx + edgeW + (Math.random() - 0.5) * 2 * treeScale,
            y: ey + (Math.random() - 0.5) * 2 * treeScale,
            color: lightColors[Math.floor(Math.random() * lightColors.length)],
            phase: Math.random() * Math.PI * 2,
            radius: (1.5 + Math.random()) * treeScale,
          })
        }
      }

      layerTrees.push({
        x: tx, groundY: hillY, scale: treeScale, layer,
        trunkH, treeH, baseW, tiers, lights,
        snowCoverage: 0.2 + Math.random() * 0.8,  // some trees barely have snow, others full
        greenHue: -15 + Math.random() * 30,        // vary green shade
        lean: (Math.random() - 0.5) * 0.15,        // slight tilt
      })
    }
    trees.push(layerTrees)
  }
  return { trees }
}

function drawWinterLandscape(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  state: WinterState,
  christmas: boolean = false,
  time: number = 0,
) {
  ctx.save()

  const hillColor = ['#dce6f0', '#e8eff6', '#f4f7fb']
  const hillBase = [h * 0.55, h * 0.65, h * 0.78]
  const hillAmp = [h * 0.08, h * 0.1, h * 0.07]
  const hillFreq = [0.003, 0.005, 0.004]
  const hillPhase = [0, 1.5, 3.2]

  for (let layer = 0; layer < 3; layer++) {
    ctx.fillStyle = hillColor[layer]
    ctx.beginPath()
    ctx.moveTo(0, h)
    for (let x = 0; x <= w; x += 2) {
      const y = hillBase[layer]
        - Math.sin(x * hillFreq[layer] + hillPhase[layer]) * hillAmp[layer]
        - Math.sin(x * hillFreq[layer] * 2.3 + hillPhase[layer] * 1.7) * hillAmp[layer] * 0.3
      ctx.lineTo(x, y)
    }
    ctx.lineTo(w, h)
    ctx.closePath()
    ctx.fill()

    // Pre-generated pine trees
    for (const tree of state.trees[layer]) {
      drawPineTree(ctx, tree)
    }

    // Christmas lights on trees (twinkling, no glow)
    if (christmas) {
      for (const tree of state.trees[layer]) {
        for (const light of tree.lights) {
          const twinkle = 0.5 + 0.5 * Math.sin(time * 0.003 + light.phase)
          const alpha = 0.5 + 0.5 * twinkle
          // Solid bulb only
          ctx.beginPath()
          ctx.arc(light.x, light.y, light.radius, 0, Math.PI * 2)
          ctx.fillStyle = light.color + Math.round(alpha * 255).toString(16).padStart(2, '0')
          ctx.fill()
        }
      }
    }
  }

  // Foreground snow ground
  ctx.fillStyle = '#f0f4f8'
  ctx.fillRect(0, h * 0.92, w, h * 0.08)
  const snowEdge = ctx.createLinearGradient(0, h * 0.88, 0, h * 0.94)
  snowEdge.addColorStop(0, 'rgba(240,244,248,0)')
  snowEdge.addColorStop(1, '#f0f4f8')
  ctx.fillStyle = snowEdge
  ctx.fillRect(0, h * 0.88, w, h * 0.06)

  ctx.restore()
}

function drawPineTree(ctx: CanvasRenderingContext2D, t: WinterTree) {
  const { x, groundY, scale, layer, trunkH, treeH, baseW, tiers, snowCoverage, greenHue, lean } = t

  ctx.save()
  // Apply lean (tilt)
  if (lean) {
    ctx.translate(x, groundY)
    ctx.rotate(lean)
    ctx.translate(-x, -groundY)
  }

  // Trunk — varied brown
  const trunkShade = layer === 0 ? '#8a9aaa' : `hsl(${25 + greenHue * 0.3}, 25%, ${25 + layer * 8}%)`
  ctx.fillStyle = trunkShade
  ctx.fillRect(x - 2 * scale, groundY - trunkH, 4 * scale, trunkH)

  // Green foliage — varied hue
  const gBase = layer === 0 ? [80, 100, 90] : layer === 1 ? [50, 80, 60] : [35, 65, 45]
  const gR = Math.max(0, Math.min(255, gBase[0] + greenHue))
  const gG = Math.max(0, Math.min(255, gBase[1] + greenHue * 0.5))
  const gB = Math.max(0, Math.min(255, gBase[2] + greenHue * 0.3))

  for (let i = 0; i < tiers; i++) {
    const frac = i / tiers
    const tierY = groundY - trunkH - frac * treeH
    const tierW = baseW * (1 - frac * 0.6)
    const tierH = treeH / tiers * 1.4

    // Green tier
    ctx.fillStyle = `rgba(${gR},${gG},${gB},${0.7 + layer * 0.1})`
    ctx.beginPath()
    ctx.moveTo(x, tierY - tierH)
    ctx.lineTo(x - tierW, tierY)
    ctx.lineTo(x + tierW, tierY)
    ctx.closePath()
    ctx.fill()

    // Snow cap — only on some tiers based on snowCoverage
    if (Math.random() < snowCoverage) {
      const capW = 0.3 + snowCoverage * 0.5  // wider cap = more snow
      ctx.fillStyle = layer === 0 ? `rgba(200,210,225,${0.5 + snowCoverage * 0.3})` : `rgba(240,245,250,${0.5 + snowCoverage * 0.4})`
      ctx.beginPath()
      ctx.moveTo(x, tierY - tierH)
      ctx.lineTo(x - tierW * capW, tierY - tierH * (0.2 + snowCoverage * 0.2))
      ctx.lineTo(x + tierW * capW, tierY - tierH * (0.2 + snowCoverage * 0.2))
      ctx.closePath()
      ctx.fill()
    }
  }
  ctx.restore()
}

// ── Replicant rooftops (static backdrop for rain on replicant theme) ──

interface Rooftop {
  x: number; width: number; height: number
  topShape: 'flat' | 'slant-left' | 'slant-right' | 'antenna' | 'vent' | 'step'
  topParam: number
  hasPipe: boolean
  hasRailing: boolean
}
interface RooftopState {
  rooftops: Rooftop[]
  groundY: number
}

function initRooftopState(w: number, h: number): RooftopState {
  const groundY = h * 0.65
  const rooftops: Rooftop[] = []
  const shapes: Rooftop['topShape'][] = ['flat', 'slant-left', 'slant-right', 'antenna', 'vent', 'step']

  // Near rooftops — large, at bottom
  let rx = -20
  while (rx < w + 40) {
    const rw = 40 + Math.random() * 90
    const rh = 30 + Math.random() * 80
    rooftops.push({
      x: rx, width: rw, height: rh,
      topShape: shapes[Math.floor(Math.random() * shapes.length)],
      topParam: 0.2 + Math.random() * 0.6,
      hasPipe: Math.random() > 0.6,
      hasRailing: Math.random() > 0.5,
    })
    rx += rw + 5 + Math.random() * 25
  }

  // Distant rooftops — smaller, higher up
  rx = -10
  while (rx < w + 30) {
    const rw = 20 + Math.random() * 50
    const rh = 15 + Math.random() * 40
    rooftops.push({
      x: rx, width: rw, height: rh + 60 + Math.random() * 30,  // taller = further back
      topShape: shapes[Math.floor(Math.random() * shapes.length)],
      topParam: 0.15 + Math.random() * 0.5,
      hasPipe: false,
      hasRailing: false,
    })
    rx += rw + 15 + Math.random() * 50
  }

  // Sort by height so distant (taller) ones draw behind near ones
  rooftops.sort((a, b) => b.height - a.height)

  return { rooftops, groundY }
}

function drawRooftops(ctx: CanvasRenderingContext2D, w: number, h: number, state: RooftopState) {
  ctx.save()
  const gY = state.groundY

  // Pink/red glow at horizon — behind the rooftops
  const glow = ctx.createRadialGradient(w * 0.5, gY - 20, 0, w * 0.5, gY - 20, w * 0.7)
  glow.addColorStop(0, 'rgba(255,45,123,0.20)')
  glow.addColorStop(0.3, 'rgba(255,30,100,0.12)')
  glow.addColorStop(0.6, 'rgba(180,20,80,0.06)')
  glow.addColorStop(1, 'rgba(80,10,40,0)')
  ctx.fillStyle = glow
  ctx.fillRect(0, 0, w, h)

  // Secondary wider glow — off-center
  const glow2 = ctx.createRadialGradient(w * 0.7, gY, 0, w * 0.7, gY, w * 0.5)
  glow2.addColorStop(0, 'rgba(255,60,100,0.10)')
  glow2.addColorStop(0.5, 'rgba(200,30,70,0.05)')
  glow2.addColorStop(1, 'rgba(100,10,30,0)')
  ctx.fillStyle = glow2
  ctx.fillRect(0, 0, w, h)

  // Draw each rooftop
  for (const rt of state.rooftops) {
    const baseY = gY + 10
    const topY = baseY - rt.height
    const slopeH = rt.height * rt.topParam * 0.25

    // Main building silhouette
    ctx.fillStyle = '#05070c'
    ctx.beginPath()
    ctx.moveTo(rt.x, baseY + 50)  // extend below ground
    switch (rt.topShape) {
      case 'slant-left':
        ctx.lineTo(rt.x, topY - slopeH)
        ctx.lineTo(rt.x + rt.width, topY)
        break
      case 'slant-right':
        ctx.lineTo(rt.x, topY)
        ctx.lineTo(rt.x + rt.width, topY - slopeH)
        break
      case 'antenna':
        ctx.lineTo(rt.x, topY)
        ctx.lineTo(rt.x + rt.width * 0.45, topY)
        ctx.lineTo(rt.x + rt.width * 0.48, topY - slopeH * 2)
        ctx.lineTo(rt.x + rt.width * 0.52, topY - slopeH * 2)
        ctx.lineTo(rt.x + rt.width * 0.55, topY)
        ctx.lineTo(rt.x + rt.width, topY)
        break
      case 'vent':
        ctx.lineTo(rt.x, topY)
        ctx.lineTo(rt.x + rt.width * 0.3, topY)
        ctx.lineTo(rt.x + rt.width * 0.3, topY - slopeH)
        ctx.lineTo(rt.x + rt.width * 0.55, topY - slopeH)
        ctx.lineTo(rt.x + rt.width * 0.55, topY)
        ctx.lineTo(rt.x + rt.width, topY)
        break
      case 'step':
        ctx.lineTo(rt.x, topY + slopeH)
        ctx.lineTo(rt.x + rt.width * 0.4, topY + slopeH)
        ctx.lineTo(rt.x + rt.width * 0.4, topY)
        ctx.lineTo(rt.x + rt.width, topY)
        break
      default:
        ctx.lineTo(rt.x, topY)
        ctx.lineTo(rt.x + rt.width, topY)
    }
    ctx.lineTo(rt.x + rt.width, baseY + 50)
    ctx.closePath()
    ctx.fill()

    // Rooftop edge highlight — faint neon reflection
    ctx.strokeStyle = 'rgba(255,45,123,0.08)'
    ctx.lineWidth = 1
    ctx.beginPath()
    ctx.moveTo(rt.x, topY)
    ctx.lineTo(rt.x + rt.width, topY)
    ctx.stroke()

    // Pipe / chimney
    if (rt.hasPipe) {
      ctx.fillStyle = '#05070c'
      const px = rt.x + rt.width * (0.7 + rt.topParam * 0.2)
      ctx.fillRect(px, topY - 12, 5, 12)
    }

    // Railing along edge
    if (rt.hasRailing) {
      ctx.strokeStyle = 'rgba(20,25,40,0.8)'
      ctx.lineWidth = 1
      ctx.beginPath()
      ctx.moveTo(rt.x + 3, topY - 6)
      ctx.lineTo(rt.x + rt.width - 3, topY - 6)
      ctx.stroke()
      // Railing posts
      for (let rp = rt.x + 8; rp < rt.x + rt.width - 5; rp += 15) {
        ctx.beginPath()
        ctx.moveTo(rp, topY)
        ctx.lineTo(rp, topY - 6)
        ctx.stroke()
      }
    }
  }

  // Ground below rooftops — solid dark
  ctx.fillStyle = '#05070c'
  ctx.fillRect(0, gY + 10, w, h - gY)

  ctx.restore()
}

// (Fruit, junkfood draw functions moved to CSSParticleEffect — GPU-composited)

// ── Boba (milk tea with tapioca pearls + swirling liquid + accelerometer) ──

interface BobaPearl {
  x: number; y: number; vx: number; vy: number
  r: number; shade: number; wobblePhase: number
}

interface BobaSwirl {
  cx: number; cy: number; radius: number; speed: number; phase: number; opacity: number
}

interface BobaState {
  pearls: BobaPearl[]
  swirls: BobaSwirl[]
  accelX: number   // current accelerometer tilt (-1 to 1)
  accelY: number
  mouseX: number   // mouse position for desktop repulsion
  mouseY: number
  mouseActive: boolean
  motionCleanup: (() => void) | null
}

function initBobaState(w: number, h: number, _foreground: boolean): BobaState {
  // Calculate pearl count to fill ~30% of screen bottom
  // avgR ≈ 15, pearl area ≈ π*15² ≈ 707, packing efficiency ~0.6
  const avgR = 15
  const targetArea = w * h * 0.40
  const packingEfficiency = 0.6
  const pearlArea = Math.PI * avgR * avgR
  const pearlCount = Math.min(300, Math.max(40, Math.round(targetArea * packingEfficiency / pearlArea)))

  // Spawn pearls scattered — they'll settle via gravity
  const pearls: BobaPearl[] = Array.from({ length: pearlCount }, () => {
    const r = 8 + Math.random() * 14
    return {
      x: Math.random() * w,
      y: h * 0.3 + Math.random() * h * 0.7,  // bottom 70%, will settle
      vx: 0, vy: 0,
      r,
      shade: Math.random(),
      wobblePhase: Math.random() * Math.PI * 2,
    }
  })

  const swirls: BobaSwirl[] = Array.from({ length: 6 }, () => ({
    cx: Math.random() * w,
    cy: Math.random() * h,
    radius: 80 + Math.random() * 160,
    speed: 0.0003 + Math.random() * 0.0005,
    phase: Math.random() * Math.PI * 2,
    opacity: 0.12 + Math.random() * 0.1,
  }))

  const state: BobaState = { pearls, swirls, accelX: 0, accelY: 0, mouseX: -1000, mouseY: -1000, mouseActive: false, motionCleanup: null }

  // ── Accelerometer setup ──
  // Low-pass filter + dead zone to prevent jitter from sensor noise
  const accelSmoothing = 0.15   // blend factor: 0 = ignore new data, 1 = no smoothing
  const accelDeadZone = 0.08    // ignore tilts smaller than this (normalized -1 to 1)
  let rawAccelX = 0, rawAccelY = 0

  const handleMotion = (e: DeviceMotionEvent) => {
    const ag = e.accelerationIncludingGravity
    if (!ag) return
    rawAccelX = (ag.x ?? 0) / 9.8
    rawAccelY = (ag.y ?? 0) / 9.8
    // Dead zone — ignore tiny tilts (sensor noise)
    if (Math.abs(rawAccelX) < accelDeadZone) rawAccelX = 0
    if (Math.abs(rawAccelY) < accelDeadZone) rawAccelY = 0
    // Low-pass filter — smooth out rapid changes
    state.accelX += (rawAccelX - state.accelX) * accelSmoothing
    state.accelY += (rawAccelY - state.accelY) * accelSmoothing
  }

  if (typeof window !== 'undefined' && 'DeviceMotionEvent' in window) {
    const DME = DeviceMotionEvent as unknown as {
      requestPermission?: () => Promise<string>
    }
    let permissionGranted = false

    if (typeof DME.requestPermission === 'function') {
      // iOS 13+ — needs user gesture. Listen for ANY interaction (touch, click, pointerdown).
      // Don't use { once: true } — keep retrying until permission granted in case
      // the first interaction gets consumed by scrolling or the prompt is dismissed.
      const requestPermission = () => {
        if (permissionGranted) return
        DME.requestPermission!().then((perm: string) => {
          if (perm === 'granted') {
            permissionGranted = true
            window.addEventListener('devicemotion', handleMotion)
            // Clean up gesture listeners once granted
            document.removeEventListener('touchstart', requestPermission)
            document.removeEventListener('click', requestPermission)
            document.removeEventListener('pointerdown', requestPermission)
          }
        }).catch(() => {})
      }
      document.addEventListener('touchstart', requestPermission)
      document.addEventListener('click', requestPermission)
      document.addEventListener('pointerdown', requestPermission)
      state.motionCleanup = () => {
        document.removeEventListener('touchstart', requestPermission)
        document.removeEventListener('click', requestPermission)
        document.removeEventListener('pointerdown', requestPermission)
        window.removeEventListener('devicemotion', handleMotion)
      }
    } else {
      // Android / desktop — just listen directly (no permission needed)
      window.addEventListener('devicemotion', handleMotion)
      state.motionCleanup = () => window.removeEventListener('devicemotion', handleMotion)
    }
  }

  // ── Mouse tracking for desktop repulsion ──
  const handleMouseMove = (e: MouseEvent) => {
    state.mouseX = e.clientX
    state.mouseY = e.clientY
    state.mouseActive = true
  }
  const handleMouseLeave = () => { state.mouseActive = false }
  window.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseleave', handleMouseLeave)

  const origCleanup = state.motionCleanup
  state.motionCleanup = () => {
    if (origCleanup) origCleanup()
    window.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseleave', handleMouseLeave)
  }

  return state
}

function cleanupBobaState(state: BobaState) {
  if (state.motionCleanup) state.motionCleanup()
}

function drawBoba(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  t: number,
  state: BobaState,
  foreground: boolean,
) {
  // ── Swirling milk tea streams (background only) ──
  if (!foreground) {
    ctx.save()
    for (const s of state.swirls) {
      const angle = t * s.speed + s.phase
      const sx = s.cx + Math.sin(angle * 0.7) * 40
      const sy = s.cy + Math.cos(angle * 0.5) * 30

      ctx.beginPath()
      ctx.arc(sx, sy, s.radius, angle, angle + Math.PI * 1.2)
      ctx.lineWidth = 25 + Math.sin(t * 0.001 + s.phase) * 10
      ctx.strokeStyle = `rgba(245, 230, 210, ${s.opacity})`
      ctx.lineCap = 'round'
      ctx.stroke()

      ctx.beginPath()
      ctx.arc(sx, sy, s.radius * 0.6, angle + Math.PI * 0.5, angle + Math.PI * 1.5)
      ctx.lineWidth = 18 + Math.sin(t * 0.0015 + s.phase) * 8
      ctx.strokeStyle = `rgba(180, 140, 100, ${s.opacity * 0.7})`
      ctx.stroke()
    }
    ctx.restore()
  }

  // ── Physics constants ──
  const gravity = 0.15          // settle downward
  const friction = 0.92         // heavy damping — viscous milk tea
  const accelForce = 1.0        // how strongly tilt affects pearls (reduced to prevent jitter)
  const floorBounce = 0.15      // very low bounce — pearls settle quickly
  const wallBounce = 0.2
  const mouseRadius = 100       // repulsion radius around cursor
  const mouseForce = 3.0        // repulsion strength
  const restThreshold = 0.4     // velocity below this = at rest (raised to prevent jitter)

  // ── Tapioca pearls with physics ──
  for (const p of state.pearls) {
    // Apply forces
    p.vy += gravity                                       // gravity pulls down
    p.vx += state.accelX * accelForce                     // tilt pushes sideways
    p.vy -= state.accelY * accelForce                     // tilt pushes up/down (inverted)

    // Mouse repulsion (desktop)
    if (state.mouseActive) {
      const mdx = p.x - state.mouseX
      const mdy = p.y - state.mouseY
      const mDist = Math.sqrt(mdx * mdx + mdy * mdy)
      if (mDist < mouseRadius && mDist > 0) {
        const strength = mouseForce * (1 - mDist / mouseRadius)
        p.vx += (mdx / mDist) * strength
        p.vy += (mdy / mDist) * strength
      }
    }

    // Damping (viscous liquid)
    p.vx *= friction
    p.vy *= friction

    // Check if at rest (supported and slow)
    const speed = Math.abs(p.vx) + Math.abs(p.vy)
    const supported = p.y + p.r >= h - 2 ||
      state.pearls.some(q => q !== p && q.y > p.y &&
        Math.abs(q.x - p.x) < p.r + q.r * 1.1 &&
        q.y - p.y < p.r + q.r + 3)
    const atRest = speed < restThreshold && supported

    if (atRest) {
      // Completely frozen — no movement, no velocity, no jitter
      p.vx = 0
      p.vy = 0
    } else {
      // Move
      p.x += p.vx
      p.y += p.vy

      // Bounce off walls
      if (p.x - p.r < 0) { p.x = p.r; p.vx = Math.abs(p.vx) * wallBounce }
      if (p.x + p.r > w) { p.x = w - p.r; p.vx = -Math.abs(p.vx) * wallBounce }

      // Bounce off floor + settle
      if (p.y + p.r > h) {
        p.y = h - p.r
        p.vy = -Math.abs(p.vy) * floorBounce
        if (Math.abs(p.vy) < 0.5) p.vy = 0
      }
      // Ceiling
      if (p.y - p.r < 0) { p.y = p.r; p.vy = Math.abs(p.vy) * floorBounce }
    }

    // Pearl-pearl collision — position separation only (no velocity when settled)
    for (const q of state.pearls) {
      if (q === p) continue
      const dx = q.x - p.x, dy = q.y - p.y
      const dist = Math.sqrt(dx * dx + dy * dy)
      const minDist = p.r + q.r
      if (dist < minDist && dist > 0) {
        const overlap = (minDist - dist) * 0.5
        const nx = dx / dist, ny = dy / dist
        // Always separate overlapping pearls
        p.x -= nx * overlap
        p.y -= ny * overlap
        q.x += nx * overlap
        q.y += ny * overlap

        // Only transfer velocity if at least one pearl is moving
        const pSpeed = Math.abs(p.vx) + Math.abs(p.vy)
        const qSpeed = Math.abs(q.vx) + Math.abs(q.vy)
        if (pSpeed > restThreshold * 2 || qSpeed > restThreshold * 2) {
          const transferAmount = 0.05
          p.vx -= nx * transferAmount
          p.vy -= ny * transferAmount
          q.vx += nx * transferAmount
          q.vy += ny * transferAmount
        }
      }
    }

    // Final bounds clamp — collision separation can push pearls outside viewport
    if (p.x - p.r < 0) { p.x = p.r; p.vx = 0 }
    if (p.x + p.r > w) { p.x = w - p.r; p.vx = 0 }
    if (p.y + p.r > h) { p.y = h - p.r; p.vy = 0 }
    if (p.y - p.r < 0) { p.y = p.r; p.vy = 0 }

    // ── Draw pearl (fully opaque) ──
    const bx = p.x, by = p.y
    ctx.save()

    const baseR = Math.floor(30 + p.shade * 30)
    const baseG = Math.floor(20 + p.shade * 20)
    const baseB = Math.floor(10 + p.shade * 15)
    const grad = ctx.createRadialGradient(
      bx - p.r * 0.25, by - p.r * 0.25, p.r * 0.05,
      bx, by, p.r
    )
    grad.addColorStop(0, `rgb(${baseR + 50}, ${baseG + 40}, ${baseB + 30})`)
    grad.addColorStop(0.5, `rgb(${baseR + 10}, ${baseG + 5}, ${baseB})`)
    grad.addColorStop(1, `rgb(${Math.max(0, baseR - 15)}, ${Math.max(0, baseG - 12)}, ${Math.max(0, baseB - 8)})`)
    ctx.fillStyle = grad
    ctx.beginPath()
    ctx.arc(bx, by, p.r, 0, Math.PI * 2)
    ctx.fill()

    // Glossy highlight
    const hlGrad = ctx.createRadialGradient(
      bx - p.r * 0.3, by - p.r * 0.3, 0,
      bx - p.r * 0.3, by - p.r * 0.3, p.r * 0.45
    )
    hlGrad.addColorStop(0, 'rgba(255, 255, 245, 0.6)')
    hlGrad.addColorStop(1, 'rgba(255, 255, 245, 0)')
    ctx.fillStyle = hlGrad
    ctx.beginPath()
    ctx.arc(bx - p.r * 0.3, by - p.r * 0.3, p.r * 0.45, 0, Math.PI * 2)
    ctx.fill()

    ctx.restore()
  }
}

// ── Wasteland (BR 2049: orange haze, city silhouette, spinner cars) ──

interface Spinner {
  x: number; y: number
  speed: number        // horizontal speed (positive = moving right/away)
  size: number         // scale factor — smaller = further away
  ledColor: string     // rear LED color
  altitude: number     // slight vertical drift
  altSpeed: number
}

interface WastelandBuilding {
  x: number; width: number; height: number; hasAntenna: boolean
  topShape: 'flat' | 'slant-left' | 'slant-right' | 'peak' | 'notch' | 'step'
  topParam: number  // how dramatic the shape is (0-1)
}

interface WastelandState {
  buildings: WastelandBuilding[]
  spinners: Spinner[]
  dustParticles: Particle[]
  hazeDrift: number
}

function initWastelandState(w: number, h: number, foreground: boolean): WastelandState {
  // Skyline — jagged silhouette of ruined buildings
  const buildingCount = Math.floor(w / 35) + 8
  // Fewer peaks/needles — weight toward blocky shapes
  const topShapes: WastelandBuilding['topShape'][] = ['flat', 'flat', 'slant-left', 'slant-right', 'notch', 'step', 'step', 'peak']
  const buildings: WastelandBuilding[] = []
  let bx = -10  // start just off-screen left
  while (bx < w + 50) {
    const bw = 30 + Math.random() * 50
    buildings.push({
      x: bx,
      width: bw,
      height: h * 0.06 + Math.random() * h * 0.28,
      hasAntenna: Math.random() > 0.88,
      topShape: topShapes[Math.floor(Math.random() * topShapes.length)],
      topParam: 0.15 + Math.random() * 0.4,
    })
    bx += bw  // next building starts exactly where this one ends — no gaps
  }

  // Flying spinner cars
  const spinners: Spinner[] = foreground ? [] : Array.from({ length: 4 }, () => spawnSpinner(w, h))

  // Dust / haze particles
  const dustCount = foreground ? 20 : 60
  const dustParticles: Particle[] = Array.from({ length: dustCount }, () => ({
    x: Math.random() * w,
    y: Math.random() * h,
    vx: 0.2 + Math.random() * 0.5,
    vy: (Math.random() - 0.5) * 0.15,
    size: foreground ? 2 + Math.random() * 5 : 1 + Math.random() * 3,
    opacity: foreground ? 0.06 + Math.random() * 0.1 : 0.03 + Math.random() * 0.06,
    phase: Math.random() * Math.PI * 2,
  }))

  return { buildings, spinners, dustParticles, hazeDrift: 0 }
}

function spawnSpinner(w: number, h: number): Spinner {
  const size = 0.3 + Math.random() * 0.7  // depth: 0.3 = far, 1.0 = close
  const goingRight = Math.random() > 0.3   // mostly moving away (right)
  return {
    x: goingRight ? -60 : w + 60,
    y: h * 0.15 + Math.random() * h * 0.4,  // in the sky above skyline
    speed: (goingRight ? 1 : -1) * (0.5 + size * 1.5 + Math.random()),  // closer = faster
    size,
    ledColor: Math.random() > 0.5 ? 'rgba(255,60,30,A)' : 'rgba(255,160,40,A)',
    altitude: 0,
    altSpeed: (Math.random() - 0.5) * 0.02,
  }
}

function drawWasteland(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  t: number,
  state: WastelandState,
  foreground: boolean,
) {
  // ── Background layer ──
  if (!foreground) {
    ctx.save()

    // Animated haze drift
    state.hazeDrift += 0.0002

    // Draw city skyline silhouette — fully opaque
    const skylineY = h * 0.62  // horizon line — lower to show more sky
    ctx.fillStyle = '#1a0c02'
    for (const b of state.buildings) {
      const baseY = skylineY + (h - skylineY) * 0.05
      const topY = baseY - b.height
      const slopeH = b.height * b.topParam * 0.4  // how much the top varies

      // Draw angular building shape
      ctx.beginPath()
      ctx.moveTo(b.x, baseY + 20)  // bottom left
      switch (b.topShape) {
        case 'slant-left':
          ctx.lineTo(b.x, topY - slopeH)
          ctx.lineTo(b.x + b.width, topY)
          break
        case 'slant-right':
          ctx.lineTo(b.x, topY)
          ctx.lineTo(b.x + b.width, topY - slopeH)
          break
        case 'peak':
          ctx.lineTo(b.x, topY)
          ctx.lineTo(b.x + b.width * 0.3, topY - slopeH)
          ctx.lineTo(b.x + b.width * 0.7, topY - slopeH)  // wide flat-top peak, not a needle
          ctx.lineTo(b.x + b.width, topY)
          break
        case 'notch':
          ctx.lineTo(b.x, topY)
          ctx.lineTo(b.x + b.width * 0.35, topY)
          ctx.lineTo(b.x + b.width * 0.35, topY + slopeH * 0.6)
          ctx.lineTo(b.x + b.width * 0.65, topY + slopeH * 0.6)
          ctx.lineTo(b.x + b.width * 0.65, topY)
          ctx.lineTo(b.x + b.width, topY)
          break
        case 'step':
          ctx.lineTo(b.x, topY)
          ctx.lineTo(b.x + b.width * 0.5, topY)
          ctx.lineTo(b.x + b.width * 0.5, topY + slopeH * 0.5)
          ctx.lineTo(b.x + b.width, topY + slopeH * 0.5)
          break
        default: // flat
          ctx.lineTo(b.x, topY)
          ctx.lineTo(b.x + b.width, topY)
      }
      ctx.lineTo(b.x + b.width, baseY + 20)  // bottom right
      ctx.closePath()
      ctx.fill()

      // Antenna spire — only on flat/step tops (not peaks which already have points)
      if (b.hasAntenna && b.topShape !== 'peak') {
        const antennaX = b.x + b.width * 0.45
        ctx.fillRect(antennaX - 1.5, topY - b.height * 0.15, 3, b.height * 0.15)
      }

      // Window glows — dim orange squares (sparse)
      const windowSeed = b.x * 7 + b.width * 13  // deterministic per building
      const windowCount = Math.floor(b.height / 30)
      for (let wi = 0; wi < windowCount; wi++) {
        const wHash = Math.sin(windowSeed + wi * 17.3) * 0.5 + 0.5
        if (wHash > 0.7) {  // ~30% of windows lit
          const wx = b.x + 3 + (wHash * 97 % 1) * Math.max(b.width - 8, 1)
          const wy = topY + 8 + wi * 28
          if (wy < baseY - 5) {
            ctx.fillStyle = `rgba(255,160,40,${0.12 + wHash * 0.15})`
            ctx.fillRect(wx, wy, 3, 2)
            ctx.fillStyle = '#1a0c02'
          }
        }
      }
    }

    // Curvy road leading toward the city (perspective vanishing point)
    const roadVanishX = w * 0.5
    const roadVanishY = skylineY + h * 0.02
    ctx.fillStyle = '#1a0c02'  // same color as buildings — seamless intersection
    ctx.beginPath()
    ctx.moveTo(roadVanishX - 8, roadVanishY)
    ctx.quadraticCurveTo(w * 0.35, h * 0.82, w * 0.05, h + 10)
    ctx.lineTo(w * 0.65, h + 10)
    ctx.quadraticCurveTo(w * 0.65, h * 0.82, roadVanishX + 8, roadVanishY)
    ctx.closePath()
    ctx.fill()

    // ── Flying spinner cars ──
    for (const sp of state.spinners) {
      sp.x += sp.speed
      sp.altitude += sp.altSpeed
      if (Math.abs(sp.altitude) > 3) sp.altSpeed *= -1
      const sy = sp.y + sp.altitude + Math.sin(t * 0.002 + sp.x * 0.01) * 2

      // Car body — dark silhouette, direction-aware
      const carW = 12 * sp.size
      const carH = 4 * sp.size
      const dir = sp.speed > 0 ? 1 : -1  // 1 = going right, -1 = going left
      // Front is in the direction of travel, rear is opposite
      const frontX = sp.x + carW * 0.5 * dir
      const rearX = sp.x - carW * dir

      // Car body — lighter than buildings so it's visible in front of skyline
      ctx.fillStyle = `rgba(80,50,30,${0.6 + sp.size * 0.4})`

      // Wedge shape: tapered at front, blunt at rear
      ctx.beginPath()
      ctx.moveTo(rearX, sy - carH * 0.3)           // rear top
      ctx.lineTo(frontX, sy - carH * 0.8)           // front top (tapered)
      ctx.lineTo(frontX, sy + carH * 0.3)           // front bottom
      ctx.lineTo(rearX, sy + carH * 0.5)            // rear bottom
      ctx.closePath()
      ctx.fill()

      // Subtle edge highlight so body reads against dark bg
      ctx.strokeStyle = `rgba(140,90,50,${0.3 * sp.size})`
      ctx.lineWidth = 0.5
      ctx.stroke()

      // Rear LEDs — always at the back of the vehicle
      const ledA = 0.7 + sp.size * 0.3 + Math.sin(t * 0.01) * 0.1
      const ledSize = 2 + sp.size * 3

      // Two tail lights at rearX
      const ledGrad1 = ctx.createRadialGradient(
        rearX, sy - carH * 0.1, 0,
        rearX, sy - carH * 0.1, ledSize * 4
      )
      ledGrad1.addColorStop(0, sp.ledColor.replace('A', String(ledA)))
      ledGrad1.addColorStop(0.3, sp.ledColor.replace('A', String(ledA * 0.4)))
      ledGrad1.addColorStop(1, sp.ledColor.replace('A', '0'))
      ctx.fillStyle = ledGrad1
      ctx.beginPath()
      ctx.arc(rearX, sy - carH * 0.1, ledSize * 4, 0, Math.PI * 2)
      ctx.fill()

      const ledGrad2 = ctx.createRadialGradient(
        rearX, sy + carH * 0.2, 0,
        rearX, sy + carH * 0.2, ledSize * 3
      )
      ledGrad2.addColorStop(0, sp.ledColor.replace('A', String(ledA * 0.8)))
      ledGrad2.addColorStop(0.4, sp.ledColor.replace('A', String(ledA * 0.3)))
      ledGrad2.addColorStop(1, sp.ledColor.replace('A', '0'))
      ctx.fillStyle = ledGrad2
      ctx.beginPath()
      ctx.arc(rearX, sy + carH * 0.2, ledSize * 3, 0, Math.PI * 2)
      ctx.fill()

      // LED core — bright white-orange center
      ctx.fillStyle = `rgba(255,220,180,${ledA * 0.9})`
      ctx.beginPath()
      ctx.arc(rearX, sy, ledSize * 0.8, 0, Math.PI * 2)
      ctx.fill()

      // Trail streak behind (opposite direction of travel)
      const trailLen = 20 + Math.abs(sp.speed) * 8
      if (Math.abs(sp.speed) > 0.5) {
        const trailEndX = rearX - dir * trailLen  // trail extends opposite to movement
        const trailGrad = ctx.createLinearGradient(trailEndX, sy, rearX, sy)
        trailGrad.addColorStop(0, sp.ledColor.replace('A', '0'))
        trailGrad.addColorStop(1, sp.ledColor.replace('A', String(ledA * 0.15)))
        ctx.strokeStyle = trailGrad
        ctx.lineWidth = ledSize * 1.5
        ctx.beginPath()
        ctx.moveTo(rearX, sy)
        ctx.lineTo(trailEndX, sy)
        ctx.stroke()
      }

      // Respawn when off screen
      if ((sp.speed > 0 && sp.x > w + 80) || (sp.speed < 0 && sp.x < -80)) {
        Object.assign(sp, spawnSpinner(w, h))
      }
    }

    ctx.restore()
  }

  // ── Dust particles (both layers) ──
  ctx.save()
  for (const d of state.dustParticles) {
    const flicker = 0.7 + Math.sin((d.phase ?? 0) + t * 0.003) * 0.3
    ctx.fillStyle = `rgba(220,160,80,${d.opacity * flicker})`
    ctx.beginPath()
    ctx.arc(d.x, d.y, d.size, 0, Math.PI * 2)
    ctx.fill()

    // Drift rightward (wind)
    d.x += (d.vx ?? 0.3)
    d.y += (d.vy ?? 0) + Math.sin((d.phase ?? 0) + t * 0.001) * 0.1
    if (d.x > w + 10) { d.x = -10; d.y = Math.random() * h }
    if (d.y < -10 || d.y > h + 10) { d.y = Math.random() * h }
  }
  ctx.restore()
}

// ── Fireworks (New Year: cityscape + water + fireworks) ──

interface FireworkShell {
  x: number; y: number; targetY: number
  vx: number; vy: number
  color: string; phase: 'rising' | 'exploding' | 'fading'
  sparks: { x: number; y: number; vx: number; vy: number; life: number }[]
  life: number; maxLife: number
}

interface FireworksState {
  shells: FireworkShell[]
  buildings: { x: number; w: number; h: number }[]
  nextLaunch: number
}

function initFireworksState(w: number, h: number): FireworksState {
  // City skyline
  const buildings: FireworksState['buildings'] = []
  let bx = 0
  while (bx < w) {
    const bw = 20 + Math.random() * 40
    buildings.push({ x: bx, w: bw, h: h * (0.08 + Math.random() * 0.2) })
    bx += bw
  }
  return { shells: [], buildings, nextLaunch: 30 }
}

function launchFirework(w: number, h: number): FireworkShell {
  const colors = ['#ff3040', '#40ff60', '#4080ff', '#ffcc00', '#ff60ff', '#00ffcc', '#ff8020', '#ffffff']
  return {
    x: w * 0.15 + Math.random() * w * 0.7,
    y: h * 0.85,
    targetY: h * (0.1 + Math.random() * 0.3),
    vx: (Math.random() - 0.5) * 1.5,
    vy: -(4 + Math.random() * 3),
    color: colors[Math.floor(Math.random() * colors.length)],
    phase: 'rising',
    sparks: [],
    life: 0,
    maxLife: 80 + Math.random() * 40,
  }
}

function drawFireworks(ctx: CanvasRenderingContext2D, w: number, h: number, t: number, state: FireworksState) {
  ctx.save()

  // Dark sky gradient
  const sky = ctx.createLinearGradient(0, 0, 0, h)
  sky.addColorStop(0, '#050510')
  sky.addColorStop(0.5, '#0a0a20')
  sky.addColorStop(1, '#101030')
  ctx.fillStyle = sky
  ctx.fillRect(0, 0, w, h)

  const waterY = h * 0.7
  const skylineY = waterY

  // City skyline silhouette
  ctx.fillStyle = '#0a0a15'
  for (const b of state.buildings) {
    ctx.fillRect(b.x, skylineY - b.h, b.w, b.h + 20)
    // Sparse lit windows
    const seed = b.x * 7 + b.w * 13
    for (let wi = 0; wi < b.h / 20; wi++) {
      if (Math.sin(seed + wi * 17) > 0.3) {
        const wx = b.x + 3 + ((Math.sin(seed + wi * 31) * 0.5 + 0.5) * Math.max(b.w - 8, 2))
        const wy = skylineY - b.h + 6 + wi * 18
        ctx.fillStyle = 'rgba(255,200,100,0.2)'
        ctx.fillRect(wx, wy, 2, 1.5)
        ctx.fillStyle = '#0a0a15'
      }
    }
  }

  // Water — dark reflective surface
  const waterGrad = ctx.createLinearGradient(0, waterY, 0, h)
  waterGrad.addColorStop(0, '#080818')
  waterGrad.addColorStop(1, '#050510')
  ctx.fillStyle = waterGrad
  ctx.fillRect(0, waterY, w, h - waterY)

  // Water ripple highlights
  ctx.strokeStyle = 'rgba(100,120,180,0.06)'
  ctx.lineWidth = 1
  for (let ry = waterY + 5; ry < h; ry += 8) {
    ctx.beginPath()
    for (let rx = 0; rx < w; rx += 4) {
      const rOff = Math.sin(rx * 0.02 + t * 0.003 + ry * 0.1) * 2
      if (rx === 0) ctx.moveTo(rx, ry + rOff)
      else ctx.lineTo(rx, ry + rOff)
    }
    ctx.stroke()
  }

  // Launch new fireworks
  state.nextLaunch--
  if (state.nextLaunch <= 0) {
    state.shells.push(launchFirework(w, h * 0.7))  // explode above waterline
    state.nextLaunch = 20 + Math.floor(Math.random() * 50)
  }

  // Update + draw shells
  for (let i = state.shells.length - 1; i >= 0; i--) {
    const s = state.shells[i]
    s.life++

    if (s.phase === 'rising') {
      s.x += s.vx
      s.y += s.vy
      s.vy += 0.03  // gravity
      // Trail
      ctx.fillStyle = 'rgba(255,200,100,0.8)'
      ctx.beginPath()
      ctx.arc(s.x, s.y, 1.5, 0, Math.PI * 2)
      ctx.fill()
      // Explode when near target or velocity flips
      if (s.y <= s.targetY || s.vy > -1) {
        s.phase = 'exploding'
        s.life = 0
        const sparkCount = 40 + Math.floor(Math.random() * 40)
        for (let si = 0; si < sparkCount; si++) {
          const angle = (si / sparkCount) * Math.PI * 2 + Math.random() * 0.3
          const speed = 1.5 + Math.random() * 3
          s.sparks.push({
            x: s.x, y: s.y,
            vx: Math.cos(angle) * speed,
            vy: Math.sin(angle) * speed,
            life: 60 + Math.random() * 40,
          })
        }
      }
    } else {
      // Exploding/fading sparks
      for (const sp of s.sparks) {
        sp.x += sp.vx
        sp.y += sp.vy
        sp.vy += 0.04  // gravity on sparks
        sp.vx *= 0.98  // drag
        sp.life--
        const alpha = Math.max(0, sp.life / 100)
        // Spark in sky
        ctx.fillStyle = s.color + Math.round(alpha * 200).toString(16).padStart(2, '0')
        ctx.beginPath()
        ctx.arc(sp.x, sp.y, 1 + alpha, 0, Math.PI * 2)
        ctx.fill()
        // Reflection in water
        if (sp.y < waterY) {
          const reflY = waterY + (waterY - sp.y) * 0.4
          const reflAlpha = alpha * 0.25
          ctx.fillStyle = s.color + Math.round(reflAlpha * 200).toString(16).padStart(2, '0')
          ctx.beginPath()
          ctx.arc(sp.x + Math.sin(t * 0.01 + sp.x * 0.1) * 2, reflY, 1, 0, Math.PI * 2)
          ctx.fill()
        }
      }
      // Remove dead sparks
      s.sparks = s.sparks.filter(sp => sp.life > 0)
      if (s.sparks.length === 0) {
        state.shells.splice(i, 1)
      }
    }
  }

  ctx.restore()
}

// ── Forest (dense trees, mushrooms, forest floor) ──

interface ForestTree {
  x: number; trunkH: number; canopyR: number; canopyY: number
  green: string; trunkColor: string; canopyLayers: number
}
interface ForestMushroom {
  x: number; y: number; capR: number; stemH: number
  isRed: boolean  // red with white spots vs brown
}
interface ForestState {
  trees: ForestTree[]
  mushrooms: ForestMushroom[]
  groundFerns: { x: number; y: number; size: number }[]
}

function initForestState(w: number, h: number): ForestState {
  const groundY = h * 0.75
  // Dense trees across the width
  const trees: ForestTree[] = []
  let tx = -20
  while (tx < w + 40) {
    const canopyR = 25 + Math.random() * 45
    const trunkH = 40 + Math.random() * 60
    trees.push({
      x: tx + (Math.random() - 0.5) * 15,
      trunkH,
      canopyR,
      canopyY: groundY - trunkH - canopyR * 0.5,
      green: `hsl(${100 + Math.random() * 50}, ${40 + Math.random() * 30}%, ${18 + Math.random() * 20}%)`,
      trunkColor: `hsl(${20 + Math.random() * 15}, ${30 + Math.random() * 20}%, ${18 + Math.random() * 12}%)`,
      canopyLayers: 2 + Math.floor(Math.random() * 2),
    })
    tx += canopyR * 0.9 + Math.random() * 20  // slightly overlapping canopies
  }

  // Red mushrooms + brown mushrooms on forest floor
  const mushrooms: ForestMushroom[] = Array.from({ length: 12 + Math.floor(Math.random() * 8) }, () => ({
    x: Math.random() * w,
    y: groundY - 2 + Math.random() * (h - groundY) * 0.3,
    capR: 4 + Math.random() * 8,
    stemH: 5 + Math.random() * 8,
    isRed: Math.random() > 0.4,  // 60% red
  }))

  // Ground ferns
  const groundFerns = Array.from({ length: 20 + Math.floor(Math.random() * 15) }, () => ({
    x: Math.random() * w,
    y: groundY - 2 + Math.random() * 8,
    size: 6 + Math.random() * 10,
  }))

  return { trees, mushrooms, groundFerns }
}

function drawForest(ctx: CanvasRenderingContext2D, w: number, h: number, state: ForestState) {
  ctx.save()
  const groundY = h * 0.75

  // Dark forest floor gradient
  const floorGrad = ctx.createLinearGradient(0, groundY - 10, 0, h)
  floorGrad.addColorStop(0, '#1a2810')
  floorGrad.addColorStop(1, '#0e1a08')
  ctx.fillStyle = floorGrad
  ctx.fillRect(0, groundY - 10, w, h - groundY + 10)

  // Trunks first (behind canopy)
  for (const t of state.trees) {
    ctx.fillStyle = t.trunkColor
    const tw = 4 + t.canopyR * 0.12
    ctx.fillRect(t.x - tw / 2, groundY - t.trunkH, tw, t.trunkH + 5)
  }

  // Canopy layers — overlapping circles
  for (const t of state.trees) {
    for (let c = 0; c < t.canopyLayers; c++) {
      const cx = t.x + (Math.sin(c * 2.5 + t.x) * t.canopyR * 0.3)
      const cy = t.canopyY + c * t.canopyR * 0.2
      const cr = t.canopyR * (1 - c * 0.15)
      ctx.fillStyle = t.green
      ctx.beginPath()
      ctx.arc(cx, cy, cr, 0, Math.PI * 2)
      ctx.fill()
    }
  }

  // Ferns — simple frond shapes
  for (const f of state.groundFerns) {
    ctx.fillStyle = `hsl(${110 + Math.random() * 20}, 45%, 22%)`
    for (let side = -1; side <= 1; side += 2) {
      for (let i = 0; i < 3; i++) {
        ctx.beginPath()
        ctx.ellipse(f.x + side * (3 + i * 3), f.y - i * 2, f.size * 0.3, f.size * 0.8, side * 0.3, 0, Math.PI * 2)
        ctx.fill()
      }
    }
  }

  // Mushrooms
  for (const m of state.mushrooms) {
    // Stem
    ctx.fillStyle = '#e8dcc8'
    ctx.fillRect(m.x - m.capR * 0.25, m.y - m.stemH, m.capR * 0.5, m.stemH)
    // Cap
    ctx.beginPath()
    ctx.arc(m.x, m.y - m.stemH, m.capR, Math.PI, 0)
    ctx.fillStyle = m.isRed ? '#cc2020' : '#8a6040'
    ctx.fill()
    // Spots on red mushrooms
    if (m.isRed) {
      ctx.fillStyle = '#f0e8d8'
      for (let s = 0; s < 3; s++) {
        const sx = m.x + (s - 1) * m.capR * 0.4
        const sy = m.y - m.stemH - m.capR * (0.3 + s * 0.15)
        ctx.beginPath()
        ctx.arc(sx, sy, m.capR * 0.12, 0, Math.PI * 2)
        ctx.fill()
      }
    }
  }

  ctx.restore()
}

// ── Main component ─────────────────────────────────────────────────────────────

export function ParticleEffect({ effect, enabled, seasonal, foreground = false }: Props) {
  const activeEffect: EffectName = !enabled
    ? 'none'
    : seasonal
      ? seasonalEffect()
      : effect

  // Snow: canvas landscape background + CSS snowflakes on top
  if (activeEffect === 'snow') {
    return <>
      {!foreground && <CanvasParticleEffect activeEffect="snow-landscape" foreground={false} />}
      <CSSParticleEffect effect="snow" foreground={foreground} />
    </>
  }

  // Delegate emoji-based effects to CSS component (GPU-composited, zero JS per frame)
  if (CSS_EFFECTS.has(activeEffect)) {
    return <CSSParticleEffect effect={activeEffect as 'leaves' | 'snow' | 'fruit' | 'junkfood' | 'sakura' | 'hearts' | 'cactus'} foreground={foreground} />
  }

  return <CanvasParticleEffect activeEffect={activeEffect} foreground={foreground} />
}

/** Canvas-based effects (complex rendering: rain, fireflies, stars, embers, flames, water, boba, clouds, warzone, digital-rain) */
function CanvasParticleEffect({ activeEffect, foreground }: { activeEffect: EffectName, foreground: boolean }) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const rafRef = useRef<number>(0)

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
      rainColumns = initRainColumns(canvas.width, canvas.height, foreground ? 'fg' : 'bg')
    }

    // 3D starfield has its own system — skip foreground (depth is built into z-projection)
    let stars3D: Star3D[] = []
    if (activeEffect === 'stars') {
      if (foreground) return
      stars3D = initStars3D(800, canvas.width, canvas.height)
    }

    // Flames use their own particle system with additive blending
    // Both 'flames' and 'embers' effects use flame particles
    let flameParticles: FlameParticle[] = []
    if (activeEffect === 'flames' || activeEffect === 'embers') {
      const flameCount = activeEffect === 'flames'
        ? (foreground ? 40 : 350)    // flames: more fire
        : (foreground ? 20 : 180)    // embers: flames + ember particles on top
      flameParticles = initFlameParticles(flameCount, canvas.width, canvas.height)
    }

    // Water caustics + bubbles (Voronoi-based)
    let waterState: WaterState | null = null
    if (activeEffect === 'water') {
      waterState = initWaterState(canvas.width, canvas.height, foreground)
    }

    // Boba state
    let bobaState: BobaState | null = null
    if (activeEffect === 'boba') {
      bobaState = initBobaState(canvas.width, canvas.height, foreground)
    }

    // Clouds state
    let cloudState: CloudState | null = null
    if (activeEffect === 'clouds') {
      cloudState = initCloudState(canvas.width, canvas.height, foreground)
    }

    // Warzone state (Terminator)
    let warzoneState: WarzoneState | null = null
    if (activeEffect === 'warzone') {
      warzoneState = initWarzoneState(canvas.width, canvas.height, foreground)
    }

    // Wasteland state (BR 2049)
    let wastelandState: WastelandState | null = null
    if (activeEffect === 'wasteland') {
      wastelandState = initWastelandState(canvas.width, canvas.height, foreground)
    }

    // Winter landscape (snow theme background)
    let winterState: WinterState | null = null
    const isChristmas = document.documentElement.getAttribute('data-theme') === 'christmas'
    if (activeEffect === 'snow-landscape') {
      winterState = initWinterState(canvas.width, canvas.height)
    }

    // Fireworks (New Year)
    let fireworksState: FireworksState | null = null
    if (activeEffect === 'fireworks' && !foreground) {
      fireworksState = initFireworksState(canvas.width, canvas.height)
    }

    // Forest
    let forestState: ForestState | null = null
    if (activeEffect === 'forest' && !foreground) {
      forestState = initForestState(canvas.width, canvas.height)
    }

    // Replicant rooftops (rain + replicant theme)
    let rooftopState: RooftopState | null = null
    if (activeEffect === 'rain' && !foreground) {
      const theme = document.documentElement.getAttribute('data-theme')
      if (theme === 'replicant') {
        rooftopState = initRooftopState(canvas.width, canvas.height)
      }
    }

    // Lightning state for rain effect
    let lightning: LightningState | null = null
    let nextLightningFrame = activeEffect === 'rain' && !foreground
      ? 120 + Math.floor(Math.random() * 300)  // first strike after 2-7 seconds
      : Infinity

    // Background counts (emoji effects handled by CSSParticleEffect)
    const bgCountMap: Record<string, number> = {
      rain: 250, fireflies: 90, stars: 800,
      embers: 250, 'digital-rain': 0, flames: 200, water: 0, boba: 0, clouds: 0, warzone: 0, wasteland: 0, fireworks: 0, forest: 0, none: 0,
    }
    const fgCountMap: Record<string, number> = {
      rain: 20, fireflies: 6, stars: 0,
      embers: 20, 'digital-rain': 0, flames: 15, water: 0, boba: 0, clouds: 0, warzone: 0, wasteland: 0, fireworks: 0, forest: 0, none: 0,
    }
    const countMap = foreground ? fgCountMap : bgCountMap
    const count = countMap[activeEffect] ?? 80
    const particles = count > 0 ? initParticles(count, canvas.width, canvas.height) : []

    // Foreground particles: larger + slightly more opaque for depth
    if (foreground) {
      for (const p of particles) {
        p.size *= 1.5
        p.opacity = Math.min(1, p.opacity * 1.2)
      }
    }

    // Effect-specific particle init
    if (activeEffect === 'embers') {
      for (const p of particles) {
        p.y = canvas.height * 0.7 + Math.random() * canvas.height * 0.3  // spawn in bottom 30%
        p.vy = -(Math.random() * 1.5 + 0.5)       // rise upward (faster)
        p.vx = (Math.random() - 0.5) * 0.6
        p.size = Math.random() * 3 + 1.5
      }
    }
    // (snow, leaves, sakura, fruit, junkfood handled by CSSParticleEffect)
    // (flames handled separately via FlameParticle system)

    let t = 0
    const animate = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height)
      t++

      const w = canvas.width
      const h = canvas.height

      if (activeEffect === 'digital-rain') {
        drawDigitalRain(ctx, rainColumns, w, h, t)
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

      // Flames use additive blending particle system
      if (activeEffect === 'flames') {
        drawFlames(ctx, flameParticles, w, h)
        updateFlames(flameParticles, w, h)
        rafRef.current = requestAnimationFrame(animate)
        return
      }

      // Embers: flames at bottom + ember particles rising above
      if (activeEffect === 'embers') {
        // Draw flames first (bottom layer)
        drawFlames(ctx, flameParticles, w, h)
        updateFlames(flameParticles, w, h)

        // Draw ember particles on top (reset composite after flames)
        ctx.globalCompositeOperation = 'lighter'
        for (const p of particles) {
          drawEmber(ctx, p, t)
          // Move embers upward with wobble
          p.x += p.vx + Math.sin((p.phase ?? 0) + t * 0.015) * 0.3
          p.y += p.vy
          p.opacity -= 0.001
          if (p.opacity < 0.05 || p.y < -20) {
            p.opacity = Math.random() * 0.7 + 0.3
            p.y = h + 10
            p.x = Math.random() * w
            p.vy = -(Math.random() * 1.5 + 0.5) // rise upward
          }
        }
        ctx.globalCompositeOperation = 'source-over'
        rafRef.current = requestAnimationFrame(animate)
        return
      }

      // Winter landscape (snow/christmas theme)
      if (activeEffect === 'snow-landscape' && winterState) {
        ctx.clearRect(0, 0, w, h)
        drawWinterLandscape(ctx, w, h, winterState, isChristmas, t)
        if (isChristmas) {
          // Keep animating for twinkling lights
          rafRef.current = requestAnimationFrame(animate)
        }
        // Non-christmas: static, draws once and stops
        return
      }

      // Warzone (Terminator)
      if (activeEffect === 'warzone' && warzoneState) {
        drawWarzone(ctx, w, h, t, warzoneState, foreground)
        rafRef.current = requestAnimationFrame(animate)
        return
      }

      // Wasteland (BR 2049)
      if (activeEffect === 'wasteland' && wastelandState) {
        drawWasteland(ctx, w, h, t, wastelandState, foreground)
        rafRef.current = requestAnimationFrame(animate)
        return
      }

      // Fireworks (New Year)
      if (activeEffect === 'fireworks' && fireworksState) {
        drawFireworks(ctx, w, h, t, fireworksState)
        rafRef.current = requestAnimationFrame(animate)
        return
      }

      // Forest
      if (activeEffect === 'forest' && forestState) {
        drawForest(ctx, w, h, forestState)
        return  // static — draws once
      }

      // Clouds
      if (activeEffect === 'clouds' && cloudState) {
        drawClouds(ctx, w, h, t, cloudState)
        rafRef.current = requestAnimationFrame(animate)
        return
      }

      // Boba pearls + swirling milk tea (background only — single layer for stacking)
      if (activeEffect === 'boba') {
        if (foreground) { rafRef.current = requestAnimationFrame(animate); return }
        if (bobaState) drawBoba(ctx, w, h, t, bobaState, false)
        rafRef.current = requestAnimationFrame(animate)
        return
      }

      // Water caustics + bubbles
      if (activeEffect === 'water' && waterState) {
        drawWater(ctx, w, h, t, waterState, foreground)
        rafRef.current = requestAnimationFrame(animate)
        return
      }

      // Replicant rooftops — draw behind rain
      if (rooftopState) {
        drawRooftops(ctx, w, h, rooftopState)
      }

      for (const p of particles) {
        // Draw (emoji effects handled by CSSParticleEffect)
        switch (activeEffect) {
          case 'rain':      drawRain(ctx, p, w); break
          case 'fireflies': drawFirefly(ctx, p, t); break
          // stars handled above via 3D system
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

      // Lightning overlay for rain effect (background layer only)
      if (activeEffect === 'rain' && !foreground) {
        // Trigger new bolt?
        if (t >= nextLightningFrame && (!lightning || !lightning.active)) {
          lightning = triggerLightning(w, h, t)
          nextLightningFrame = t + 180 + Math.floor(Math.random() * 600) // 3-13s between strikes
        }
        // Draw active bolt
        if (lightning && lightning.active) {
          drawLightning(ctx, lightning, w, h, t)
        }
      }

      rafRef.current = requestAnimationFrame(animate)
    }

    rafRef.current = requestAnimationFrame(animate)

    return () => {
      cancelAnimationFrame(rafRef.current)
      window.removeEventListener('resize', resize)
      if (bobaState) cleanupBobaState(bobaState)
    }
  }, [activeEffect])

  if (activeEffect === 'none') return null

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: 'fixed',
        inset: 0,
        width: '100%', height: '100%',
        pointerEvents: 'none',
        zIndex: foreground ? 10 : 0,
        WebkitTransform: 'translateZ(0)',
        transform: 'translateZ(0)',
        willChange: 'transform',
      }}
    />
  )
}
