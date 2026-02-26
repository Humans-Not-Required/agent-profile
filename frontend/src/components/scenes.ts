/**
 * Static canvas scene draw functions extracted from ParticleEffect.tsx.
 * Scenes are rendered once (or very slowly) as a backdrop behind effects.
 * No React dependencies — pure canvas drawing.
 */

// ── Christmas lights (used by winter landscape) ─────────────────────────────

export interface ChristmasLight {
	x: number; y: number; color: string; phase: number; radius: number
}

// ── Winter Landscape ────────────────────────────────────────────────────────

export interface WinterTree {
	x: number; groundY: number; scale: number; layer: number
	trunkH: number; treeH: number; baseW: number; tiers: number
	snowCoverage: number   // 0-1: how much snow on tips (0 = bare, 1 = full caps)
	greenHue: number       // variation in green shade
	lean: number           // slight tilt (-0.1 to 0.1)
	lights: ChristmasLight[]
}

export interface WinterState {
	trees: WinterTree[][]  // per layer
}

export function initWinterState(w: number, h: number): WinterState {
	const hillBase = [h * 0.55, h * 0.65, h * 0.78]
	const hillAmp = [h * 0.08, h * 0.1, h * 0.07]
	const hillFreq = [0.003, 0.005, 0.004]
	const hillPhase = [0, 1.5, 3.2]
	const trees: WinterTree[][] = []

	// Scale tree count to screen width — ~1 tree per 90px on front layer, fewer on back
	const baseDensity = Math.max(3, Math.round(w / 90))  // e.g. 375px→4, 768px→9, 1280px→14
	for (let layer = 0; layer < 3; layer++) {
		const layerTrees: WinterTree[] = []
		const treeCount = layer === 0 ? Math.round(baseDensity * 0.5) : layer === 1 ? Math.round(baseDensity * 0.75) : baseDensity
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

export function drawWinterLandscape(
	ctx: CanvasRenderingContext2D,
	w: number,
	h: number,
	state: WinterState,
	christmas: boolean = false,
	_time: number = 0,
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

		// Christmas lights on trees (static solid bulbs, no animation)
		if (christmas) {
			for (const tree of state.trees[layer]) {
				for (const light of tree.lights) {
					ctx.beginPath()
					ctx.arc(light.x, light.y, light.radius, 0, Math.PI * 2)
					ctx.fillStyle = light.color
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

// ── Rooftops (Replicant backdrop for rain) ──────────────────────────────────

export interface Rooftop {
	x: number; width: number; height: number
	topShape: 'flat' | 'slant-left' | 'slant-right' | 'antenna' | 'vent' | 'step'
	topParam: number
	hasPipe: boolean
	hasRailing: boolean
}

export interface RooftopState {
	rooftops: Rooftop[]
	groundY: number
}

export function initRooftopState(w: number, h: number): RooftopState {
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

export function drawRooftops(ctx: CanvasRenderingContext2D, w: number, h: number, state: RooftopState) {
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

// ── Forest (dense trees, mushrooms, forest floor) ───────────────────────────

export interface ForestTree {
	x: number; trunkH: number; canopyR: number; canopyY: number
	green: string; trunkColor: string; canopyLayers: number
}

export interface ForestMushroom {
	x: number; y: number; capR: number; stemH: number
	isRed: boolean  // red with white spots vs brown
}

export interface ForestState {
	trees: ForestTree[]
	mushrooms: ForestMushroom[]
	groundFerns: { x: number; y: number; size: number }[]
}

export function initForestState(w: number, h: number): ForestState {
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

export function drawForest(ctx: CanvasRenderingContext2D, w: number, h: number, state: ForestState) {
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
