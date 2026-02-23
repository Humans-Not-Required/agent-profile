import { useEffect, useRef } from 'react'
import { CSSParticleEffect } from './CSSParticleEffect'

export type EffectName = 'snow' | 'leaves' | 'rain' | 'fireflies' | 'stars' | 'sakura' | 'embers' | 'digital-rain' | 'flames' | 'water' | 'boba' | 'clouds' | 'fruit' | 'junkfood' | 'warzone' | 'hearts' | 'cactus' | 'none'

// Effects that use GPU-composited CSS animations instead of canvas
const CSS_EFFECTS = new Set<EffectName>(['leaves', 'snow', 'fruit', 'junkfood', 'sakura', 'hearts', 'cactus'])

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
    for (let i = 0; i < 7; i++) groups.push(makeCloudGroup(w, h, 'far'))
    // Mid layer
    for (let i = 0; i < 5; i++) groups.push(makeCloudGroup(w, h, 'mid'))
    // Near layer: big, fast, more opaque
    for (let i = 0; i < 4; i++) groups.push(makeCloudGroup(w, h, 'near'))
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

// ── Warzone (Terminator: laser sweeps, searchlights, sparks, red eye scan) ──

interface Searchlight {
  x: number; angle: number; sweepSpeed: number; width: number; alpha: number
}

interface CrossfireLaser {
  fromLeft: boolean     // true = shoots from left edge, false = right edge
  y: number             // vertical position
  angle: number         // slight angle (mostly horizontal)
  alpha: number         // current brightness (flashes then fades)
  width: number
  color: string
  life: number          // frames remaining
  maxLife: number
}

interface GroundFire {
  x: number           // center x position
  width: number       // fire width
  intensity: number   // 0-1, current brightness
  targetIntensity: number  // fading toward this
  phase: number       // animation phase offset
}

interface WarzoneState {
  searchlights: Searchlight[]
  crossfireLasers: CrossfireLaser[]
  laserCooldown: number
  groundFires: GroundFire[]
  fireCooldown: number
  sparks: Particle[]
  redEyePhase: number
  flashTimer: number
  flashAlpha: number
}

const LASER_COLORS = [
  'rgba(255,20,40,A)',   // red
  'rgba(60,140,255,A)',  // blue
  'rgba(180,60,255,A)',  // purple
  'rgba(255,100,20,A)',  // orange
  'rgba(40,255,180,A)',  // green
]

function spawnCrossfireLaser(w: number, h: number): CrossfireLaser {
  const fromLeft = Math.random() > 0.5
  return {
    fromLeft,
    y: h * 0.15 + Math.random() * h * 0.7,  // middle 70% of screen
    angle: (Math.random() - 0.5) * 0.15,      // slight angle, mostly horizontal
    alpha: 0.7 + Math.random() * 0.3,          // bright flash
    width: 1.5 + Math.random() * 2,
    color: LASER_COLORS[Math.floor(Math.random() * LASER_COLORS.length)],
    life: 8 + Math.floor(Math.random() * 15),  // short burst: 8-22 frames
    maxLife: 0,  // set below
  }
}

function initWarzoneState(w: number, h: number, foreground: boolean): WarzoneState {
  // Searchlights from the SKY, sweeping downward to search the ground
  const searchlights: Searchlight[] = foreground ? [] : Array.from({ length: 3 }, () => ({
    x: w * 0.15 + Math.random() * w * 0.7,  // spread across top
    angle: Math.PI / 2 + (Math.random() - 0.5) * 0.4,  // pointing downward
    sweepSpeed: 0.001 + Math.random() * 0.002,
    width: 35 + Math.random() * 50,
    alpha: 0.05 + Math.random() * 0.04,
  }))

  // Sparks — small bright particles
  const sparkCount = foreground ? 15 : 80
  const sparks: Particle[] = Array.from({ length: sparkCount }, () => ({
    x: Math.random() * w,
    y: h * 0.5 + Math.random() * h * 0.5,
    vx: (Math.random() - 0.5) * 2,
    vy: -(Math.random() * 2 + 0.5),
    size: foreground ? 2 + Math.random() * 3 : 1 + Math.random() * 2,
    opacity: Math.random() * 0.8 + 0.2,
    phase: Math.random() * Math.PI * 2,
  }))

  // Ground fires — intermittent flames along the bottom
  const groundFires: GroundFire[] = foreground ? [] : Array.from({ length: 5 }, (_, i) => ({
    x: w * 0.1 + (i / 4) * w * 0.8,  // spread across bottom
    width: 40 + Math.random() * 60,
    intensity: 0,
    targetIntensity: 0,
    phase: Math.random() * Math.PI * 2,
  }))

  return { searchlights, crossfireLasers: [], laserCooldown: 60, groundFires, fireCooldown: 30, sparks, redEyePhase: 0, flashTimer: 0, flashAlpha: 0 }
}

function drawWarzone(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  t: number,
  state: WarzoneState,
  foreground: boolean,
) {
  // ── Background: searchlights from sky + crossfire lasers ──
  if (!foreground) {
    ctx.save()

    // Searchlights — from the sky, sweeping down to search the ground
    for (const sl of state.searchlights) {
      const sweep = Math.sin(t * sl.sweepSpeed) * 0.6
      const angle = sl.angle + sweep
      const beamLen = h * 1.5
      const endX = sl.x + Math.cos(angle) * beamLen
      const endY = Math.sin(angle) * beamLen

      const grad = ctx.createLinearGradient(sl.x, 0, endX, endY)
      grad.addColorStop(0, `rgba(180,200,230,${sl.alpha})`)
      grad.addColorStop(0.3, `rgba(140,170,210,${sl.alpha * 0.5})`)
      grad.addColorStop(1, 'rgba(140,170,210,0)')

      ctx.beginPath()
      ctx.moveTo(sl.x - sl.width / 2, 0)    // originates from top
      ctx.lineTo(endX - sl.width * 3, endY)
      ctx.lineTo(endX + sl.width * 3, endY)
      ctx.lineTo(sl.x + sl.width / 2, 0)
      ctx.closePath()
      ctx.fillStyle = grad
      ctx.fill()
    }

    // Crossfire lasers — occasional horizontal flashes from both sides
    state.laserCooldown--
    if (state.laserCooldown <= 0) {
      // Spawn 1-3 lasers in a burst
      const burstCount = 1 + Math.floor(Math.random() * 3)
      for (let i = 0; i < burstCount; i++) {
        const laser = spawnCrossfireLaser(w, h)
        laser.maxLife = laser.life
        state.crossfireLasers.push(laser)
      }
      state.laserCooldown = 40 + Math.floor(Math.random() * 120) // 0.7-2.7s between bursts
    }

    // Draw + update crossfire lasers
    ctx.globalCompositeOperation = 'lighter'
    for (let i = state.crossfireLasers.length - 1; i >= 0; i--) {
      const laser = state.crossfireLasers[i]
      laser.life--
      if (laser.life <= 0) { state.crossfireLasers.splice(i, 1); continue }

      // Flash envelope: bright start, quick fade
      const lifeRatio = laser.life / laser.maxLife
      const envelope = lifeRatio > 0.7 ? 1.0 : lifeRatio / 0.7  // instant on, fade out
      const a = laser.alpha * envelope

      const startX = laser.fromLeft ? -10 : w + 10
      const endX = laser.fromLeft ? w + 10 : -10
      const startY = laser.y - Math.tan(laser.angle) * (laser.fromLeft ? 0 : w)
      const endY = laser.y + Math.tan(laser.angle) * (laser.fromLeft ? w : 0)

      // Bright core
      ctx.beginPath()
      ctx.moveTo(startX, startY)
      ctx.lineTo(endX, endY)
      ctx.strokeStyle = laser.color.replace('A', String(a))
      ctx.lineWidth = laser.width
      ctx.stroke()

      // Wide glow
      ctx.strokeStyle = laser.color.replace('A', String(a * 0.25))
      ctx.lineWidth = laser.width * 6
      ctx.stroke()
    }
    ctx.globalCompositeOperation = 'source-over'

    // Red eye scan line — horizontal sweep across screen
    state.redEyePhase += 0.008
    const eyeY = h * 0.3 + Math.sin(state.redEyePhase) * h * 0.25
    const eyeGrad = ctx.createLinearGradient(0, eyeY - 2, 0, eyeY + 2)
    eyeGrad.addColorStop(0, 'rgba(255,16,32,0)')
    eyeGrad.addColorStop(0.5, `rgba(255,16,32,${0.08 + Math.sin(t * 0.003) * 0.04})`)
    eyeGrad.addColorStop(1, 'rgba(255,16,32,0)')
    ctx.fillStyle = eyeGrad
    ctx.fillRect(0, eyeY - 15, w, 30)

    // Ground fires — intermittent flames along the bottom
    state.fireCooldown--
    if (state.fireCooldown <= 0) {
      // Randomly ignite or extinguish fires
      const fire = state.groundFires[Math.floor(Math.random() * state.groundFires.length)]
      fire.targetIntensity = fire.targetIntensity > 0.3 ? 0 : 0.4 + Math.random() * 0.6
      state.fireCooldown = 30 + Math.floor(Math.random() * 90)
    }

    ctx.globalCompositeOperation = 'lighter'
    for (const fire of state.groundFires) {
      // Ease toward target
      fire.intensity += (fire.targetIntensity - fire.intensity) * 0.03
      if (fire.intensity < 0.02) { fire.intensity = 0; continue }

      const flicker = Math.sin(t * 0.015 + fire.phase) * 0.15 + Math.sin(t * 0.037 + fire.phase * 2) * 0.1
      const a = fire.intensity * (0.75 + flicker)
      const flameH = (60 + fire.width * 0.8) * fire.intensity

      // Outer glow
      const outerGrad = ctx.createRadialGradient(fire.x, h, 0, fire.x, h - flameH * 0.3, fire.width * 1.5)
      outerGrad.addColorStop(0, `rgba(255,80,20,${a * 0.3})`)
      outerGrad.addColorStop(1, 'rgba(255,40,10,0)')
      ctx.fillStyle = outerGrad
      ctx.fillRect(fire.x - fire.width * 1.5, h - flameH * 1.5, fire.width * 3, flameH * 1.5)

      // Main flame body
      const grad = ctx.createLinearGradient(fire.x, h, fire.x, h - flameH)
      grad.addColorStop(0, `rgba(255,200,60,${a})`)
      grad.addColorStop(0.2, `rgba(255,120,20,${a * 0.9})`)
      grad.addColorStop(0.5, `rgba(255,50,10,${a * 0.6})`)
      grad.addColorStop(0.8, `rgba(180,20,5,${a * 0.3})`)
      grad.addColorStop(1, 'rgba(100,10,0,0)')

      // Draw flame shape with bezier curves
      ctx.beginPath()
      const halfW = fire.width / 2
      const sway1 = Math.sin(t * 0.02 + fire.phase) * halfW * 0.3
      const sway2 = Math.sin(t * 0.03 + fire.phase + 1) * halfW * 0.4
      ctx.moveTo(fire.x - halfW, h)
      ctx.bezierCurveTo(
        fire.x - halfW * 0.6, h - flameH * 0.4,
        fire.x + sway1 - halfW * 0.2, h - flameH * 0.7,
        fire.x + sway2, h - flameH
      )
      ctx.bezierCurveTo(
        fire.x - sway1 + halfW * 0.2, h - flameH * 0.7,
        fire.x + halfW * 0.6, h - flameH * 0.4,
        fire.x + halfW, h
      )
      ctx.closePath()
      ctx.fillStyle = grad
      ctx.fill()
    }
    ctx.globalCompositeOperation = 'source-over'

    // Occasional explosion flash
    state.flashTimer++
    if (state.flashTimer > 300 + Math.random() * 400) {
      state.flashAlpha = 0.15 + Math.random() * 0.1
      state.flashTimer = 0
    }
    if (state.flashAlpha > 0) {
      const flashColors = ['rgba(255,140,40,A)', 'rgba(255,60,30,A)', 'rgba(200,180,255,A)']
      const fc = flashColors[Math.floor(Math.random() * flashColors.length)]
      ctx.fillStyle = fc.replace('A', String(state.flashAlpha))
      ctx.fillRect(0, 0, w, h)
      state.flashAlpha *= 0.92  // rapid decay
      if (state.flashAlpha < 0.005) state.flashAlpha = 0
    }

    ctx.restore()
  }

  // ── Sparks — bright points flying upward from explosions ──
  ctx.save()
  ctx.globalCompositeOperation = 'lighter'
  for (const s of state.sparks) {
    const flicker = Math.sin((s.phase ?? 0) + t * 0.01) * 0.3 + 0.7
    const r = s.size * flicker

    // Bright core
    ctx.fillStyle = `rgba(255, 200, 120, ${s.opacity * flicker})`
    ctx.beginPath()
    ctx.arc(s.x, s.y, r * 0.5, 0, Math.PI * 2)
    ctx.fill()

    // Glow
    const grad = ctx.createRadialGradient(s.x, s.y, 0, s.x, s.y, r * 3)
    grad.addColorStop(0, `rgba(255, 160, 60, ${s.opacity * flicker * 0.5})`)
    grad.addColorStop(1, 'rgba(255, 80, 20, 0)')
    ctx.fillStyle = grad
    ctx.beginPath()
    ctx.arc(s.x, s.y, r * 3, 0, Math.PI * 2)
    ctx.fill()

    // Move
    s.x += (s.vx ?? 0) + Math.sin((s.phase ?? 0) + t * 0.02) * 0.2
    s.y += (s.vy ?? 0)
    s.opacity -= 0.003
    if (s.opacity < 0.05 || s.y < -20) {
      s.opacity = Math.random() * 0.8 + 0.2
      s.y = h * 0.6 + Math.random() * h * 0.4
      s.x = Math.random() * w
      s.vy = -(Math.random() * 2 + 0.5)
      s.vx = (Math.random() - 0.5) * 2
    }
  }
  ctx.globalCompositeOperation = 'source-over'
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
  const handleMotion = (e: DeviceMotionEvent) => {
    const ag = e.accelerationIncludingGravity
    if (!ag) return
    state.accelX = (ag.x ?? 0) / 9.8
    state.accelY = (ag.y ?? 0) / 9.8
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
  const accelForce = 2.0        // how strongly tilt affects pearls
  const floorBounce = 0.15      // very low bounce — pearls settle quickly
  const wallBounce = 0.2
  const mouseRadius = 100       // repulsion radius around cursor
  const mouseForce = 3.0        // repulsion strength
  const restThreshold = 0.15    // velocity below this = at rest

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

    // Settle to rest — stop micro-jittering
    if (Math.abs(p.vx) < restThreshold && Math.abs(p.vy) < restThreshold) {
      // Only fully rest if supported (on floor or on another pearl)
      const supported = p.y + p.r >= h - 1 ||
        state.pearls.some(q => q !== p && q.y > p.y &&
          Math.abs(q.x - p.x) < p.r + q.r &&
          q.y - p.y < p.r + q.r + 2)
      if (supported) { p.vx = 0; p.vy = 0 }
    }

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
      if (Math.abs(p.vy) < 0.5) p.vy = 0  // stop bouncing quickly
    }
    // Ceiling
    if (p.y - p.r < 0) { p.y = p.r; p.vy = Math.abs(p.vy) * floorBounce }

    // Pearl-pearl collision (push apart, minimal velocity transfer for stacking)
    for (const q of state.pearls) {
      if (q === p) continue
      const dx = q.x - p.x, dy = q.y - p.y
      const dist = Math.sqrt(dx * dx + dy * dy)
      const minDist = p.r + q.r
      if (dist < minDist && dist > 0) {
        const overlap = (minDist - dist) * 0.5
        const nx = dx / dist, ny = dy / dist
        // Separate them
        p.x -= nx * overlap
        p.y -= ny * overlap
        q.x += nx * overlap
        q.y += ny * overlap
        // Very light velocity transfer — enough to settle, not enough to bounce forever
        const transferAmount = 0.08
        p.vx -= nx * transferAmount
        p.vy -= ny * transferAmount
        q.vx += nx * transferAmount
        q.vy += ny * transferAmount
      }
    }

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

// ── Main component ─────────────────────────────────────────────────────────────

export function ParticleEffect({ effect, enabled, seasonal, foreground = false }: Props) {
  const activeEffect: EffectName = !enabled
    ? 'none'
    : seasonal
      ? seasonalEffect()
      : effect

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

    // Background counts (emoji effects handled by CSSParticleEffect)
    const bgCountMap: Record<string, number> = {
      rain: 250, fireflies: 90, stars: 800,
      embers: 250, 'digital-rain': 0, flames: 200, water: 0, boba: 0, clouds: 0, warzone: 0, none: 0,
    }
    const fgCountMap: Record<string, number> = {
      rain: 20, fireflies: 6, stars: 0,
      embers: 20, 'digital-rain': 0, flames: 15, water: 0, boba: 0, clouds: 0, warzone: 0, none: 0,
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

      // Warzone (Terminator)
      if (activeEffect === 'warzone' && warzoneState) {
        drawWarzone(ctx, w, h, t, warzoneState, foreground)
        rafRef.current = requestAnimationFrame(animate)
        return
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
        top: 0, left: 0,
        width: '100vw', height: '100vh',
        pointerEvents: 'none',
        zIndex: foreground ? 10 : 0,
      }}
    />
  )
}
