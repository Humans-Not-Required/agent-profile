import { useEffect, useRef } from 'react'

export type EffectName = 'snow' | 'leaves' | 'rain' | 'fireflies' | 'stars' | 'sakura' | 'embers' | 'digital-rain' | 'flames' | 'water' | 'none'

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

// ── Snowflake: Unicode glyphs (❄ ❅ ❆), slow gentle fall ──

const SNOWFLAKE_CHARS = ['❄', '❅', '❆']

function drawSnowflake(ctx: CanvasRenderingContext2D, p: Particle) {
  ctx.save()
  ctx.translate(p.x, p.y)
  ctx.rotate(p.rotation ?? 0)

  const glyph = SNOWFLAKE_CHARS[(p.color ?? 0) % SNOWFLAKE_CHARS.length]
  const fontSize = p.size * 3

  ctx.font = `${fontSize}px sans-serif`
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  ctx.fillStyle = `rgba(120, 180, 220, ${p.opacity})`
  ctx.fillText(glyph, 0, 0)

  ctx.restore()
}

// ── Leaf (Unicode characters) ──

function drawLeaf(ctx: CanvasRenderingContext2D, p: Particle) {
  ctx.save()
  ctx.translate(p.x, p.y)
  ctx.rotate(p.rotation ?? 0)
  ctx.globalAlpha = p.opacity
  ctx.font = `${p.size * 3}px serif`
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  ctx.fillText('🍁', 0, 0)
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

  // Extreme close-up columns: 1-2 very rare, very large (48-72px), fastest
  const extremeCount = Math.random() < 0.6 ? 1 : 2
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
      col.y = -col.length * col.charSize * Math.random()
      // Respawn with speed proportional to size (closer = faster) + randomness
      if (col.charSize > 40) {
        col.speed = 5 + Math.random() * 4             // extreme: 5–9
      } else if (col.charSize > 20) {
        col.speed = 3.5 + Math.random() * 2.5         // foreground: 3.5–6
      } else {
        // Background: depth-based — larger chars faster
        const depth = (col.charSize - 10) / 8          // 0–1 from charSize 10–18
        const base = 0.6 + depth * 2.4
        col.speed = Math.max(0.3, base + (Math.random() - 0.5) * 1.2)
      }
      col.length = col.charSize > 40
        ? Math.floor(Math.random() * 5) + 3
        : col.charSize > 20
          ? Math.floor(Math.random() * 8) + 5
          : Math.floor(Math.random() * 12) + 8
    }
  }
}

// ── Water caustics + bubbles ──

interface Bubble {
  x: number; y: number; r: number; speed: number; wobblePhase: number; opacity: number
}

function initBubbles(count: number, w: number, h: number): Bubble[] {
  return Array.from({ length: count }, () => ({
    x: Math.random() * w,
    y: h + Math.random() * h,
    r: 1.5 + Math.random() * 4,
    speed: 0.3 + Math.random() * 0.8,
    wobblePhase: Math.random() * Math.PI * 2,
    opacity: 0.15 + Math.random() * 0.35,
  }))
}

function drawWater(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  t: number,
  bubbles: Bubble[],
  foreground: boolean,
) {
  // ── Caustic light network ──
  if (!foreground) {
    ctx.save()
    ctx.globalCompositeOperation = 'lighter'

    // Multiple overlapping sine-based caustic layers
    const layers = [
      { freq: 0.008, amp: 18, speed: 0.0004, alpha: 0.07, color: '120,200,255' },
      { freq: 0.012, amp: 14, speed: -0.0006, alpha: 0.05, color: '80,180,240' },
      { freq: 0.006, amp: 22, speed: 0.0003, alpha: 0.06, color: '160,220,255' },
    ]

    for (const layer of layers) {
      ctx.beginPath()
      const phase = t * layer.speed
      // Draw a mesh of curved caustic lines (horizontal)
      for (let row = 0; row < h + 40; row += 40) {
        ctx.moveTo(0, row)
        for (let x = 0; x <= w; x += 8) {
          const y = row
            + Math.sin(x * layer.freq + phase) * layer.amp
            + Math.sin(x * layer.freq * 1.7 + phase * 1.3 + row * 0.01) * layer.amp * 0.6
          ctx.lineTo(x, y)
        }
      }
      // Vertical caustic lines
      for (let col = 0; col < w + 40; col += 40) {
        ctx.moveTo(col, 0)
        for (let y = 0; y <= h; y += 8) {
          const x = col
            + Math.sin(y * layer.freq + phase * 0.8) * layer.amp
            + Math.sin(y * layer.freq * 1.4 + phase * 1.1 + col * 0.01) * layer.amp * 0.5
          ctx.lineTo(x, y)
        }
      }
      ctx.strokeStyle = `rgba(${layer.color}, ${layer.alpha})`
      ctx.lineWidth = 1.5
      ctx.stroke()
    }

    // Bright caustic spots where lines would intersect
    const spotCount = 18
    for (let i = 0; i < spotCount; i++) {
      const sx = (Math.sin(t * 0.0002 + i * 1.7) * 0.5 + 0.5) * w
      const sy = (Math.cos(t * 0.00015 + i * 2.3) * 0.5 + 0.5) * h
      const sr = 30 + Math.sin(t * 0.001 + i) * 15
      const spotAlpha = 0.03 + Math.sin(t * 0.0008 + i * 0.9) * 0.02
      const grad = ctx.createRadialGradient(sx, sy, 0, sx, sy, sr)
      grad.addColorStop(0, `rgba(180, 230, 255, ${spotAlpha})`)
      grad.addColorStop(1, 'rgba(180, 230, 255, 0)')
      ctx.fillStyle = grad
      ctx.fillRect(sx - sr, sy - sr, sr * 2, sr * 2)
    }

    ctx.restore()
  }

  // ── Bubbles ──
  for (const b of bubbles) {
    ctx.save()
    const wobble = Math.sin(t * 0.002 + b.wobblePhase) * 1.5

    // Outer glow
    const g = ctx.createRadialGradient(b.x + wobble, b.y, b.r * 0.2, b.x + wobble, b.y, b.r)
    g.addColorStop(0, `rgba(200, 240, 255, ${b.opacity * 0.4})`)
    g.addColorStop(0.7, `rgba(160, 220, 255, ${b.opacity * 0.15})`)
    g.addColorStop(1, 'rgba(160, 220, 255, 0)')
    ctx.fillStyle = g
    ctx.beginPath()
    ctx.arc(b.x + wobble, b.y, b.r, 0, Math.PI * 2)
    ctx.fill()

    // Rim highlight
    ctx.strokeStyle = `rgba(220, 245, 255, ${b.opacity * 0.5})`
    ctx.lineWidth = 0.5
    ctx.stroke()

    // Specular dot
    ctx.fillStyle = `rgba(255, 255, 255, ${b.opacity * 0.6})`
    ctx.beginPath()
    ctx.arc(b.x + wobble - b.r * 0.3, b.y - b.r * 0.3, b.r * 0.2, 0, Math.PI * 2)
    ctx.fill()

    ctx.restore()

    // Animate
    b.y -= b.speed
    b.x += wobble * 0.02
    if (b.y < -b.r * 2) {
      b.y = h + b.r * 2 + Math.random() * 40
      b.x = Math.random() * w
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
      rainColumns = initRainColumns(canvas.width, canvas.height, foreground ? 'fg' : 'bg')
    }

    // 3D starfield has its own system — skip foreground (depth is built into z-projection)
    let stars3D: Star3D[] = []
    if (activeEffect === 'stars') {
      if (foreground) return
      stars3D = initStars3D(800, canvas.width, canvas.height)
    }

    // Flames use their own particle system with additive blending
    let flameParticles: FlameParticle[] = []
    if (activeEffect === 'flames') {
      const flameCount = foreground ? 30 : 250
      flameParticles = initFlameParticles(flameCount, canvas.width, canvas.height)
    }

    // Water caustics + bubbles
    let waterBubbles: Bubble[] = []
    if (activeEffect === 'water') {
      const bubbleCount = foreground ? 8 : 50
      waterBubbles = initBubbles(bubbleCount, canvas.width, canvas.height)
    }

    // Background counts — generous for immersion
    const bgCountMap: Record<EffectName, number> = {
      snow: 40, leaves: 100, rain: 250, fireflies: 90, stars: 800, sakura: 80,
      embers: 140, 'digital-rain': 0, flames: 200, water: 0, none: 0,
    }
    // Foreground: ~15% of background for subtle depth
    const fgCountMap: Record<EffectName, number> = {
      snow: 6, leaves: 8, rain: 20, fireflies: 6, stars: 0, sakura: 6,
      embers: 10, 'digital-rain': 0, flames: 15, water: 0, none: 0,
    }
    const countMap = foreground ? fgCountMap : bgCountMap
    const count = countMap[activeEffect] ?? 80
    const particles = count > 0 ? initParticles(count, canvas.width, canvas.height) : []

    // Foreground particles: larger + slightly more opaque for depth
    if (foreground) {
      for (const p of particles) {
        p.size *= 1.5
        p.opacity = Math.min(1, p.opacity * 1.2)  // brighter — closer = more vivid
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
        p.size = Math.random() * 4 + 2       // varied sizes for depth
        p.vy = Math.random() * 0.35 + 0.1    // very slow gentle fall
        p.vx = (Math.random() - 0.5) * 0.15  // barely any horizontal drift
        p.vr = (Math.random() - 0.5) * 0.006 // very slow rotation
        p.opacity = Math.random() * 0.08 + 0.04  // very subtle (0.04–0.12)
        p.color = Math.floor(Math.random() * 3)  // pick glyph variant
      }
    } else if (activeEffect === 'sakura') {
      for (const p of particles) {
        p.size = Math.random() * 3 + 2
        p.vy = Math.random() * 0.5 + 0.2    // gentle fall
        p.vx = Math.random() * 0.3 + 0.1    // slight lateral drift
        p.vr = (Math.random() - 0.5) * 0.015 // slow tumble
      }
    }
    // (flames handled separately via FlameParticle system)

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

      // Flames use additive blending particle system
      if (activeEffect === 'flames') {
        drawFlames(ctx, flameParticles, w, h)
        updateFlames(flameParticles, w, h)
        rafRef.current = requestAnimationFrame(animate)
        return
      }

      // Water caustics + bubbles
      if (activeEffect === 'water') {
        drawWater(ctx, w, h, t, waterBubbles, foreground)
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
