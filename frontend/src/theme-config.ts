import type { EffectName } from './components/ParticleEffect'

/** Named static canvas scenes rendered once or slowly behind effects. */
export type SceneName =
	| 'none'
	| 'winter-landscape'
	| 'winter-landscape-xmas'
	| 'rooftops'
	| 'forest'

/** Three-tier theme composition: CSS style + optional canvas scene + optional effect. */
export interface ThemeConfig {
	style: string
	scene: SceneName
	effect: EffectName
}

/**
 * Single source of truth for the three-tier theme mapping.
 *
 * - **style** — CSS via `[data-theme="..."]` selectors in index.css.
 * - **scene** — Static or slow-moving canvas backdrop (winter hills, rooftops, forest).
 * - **effect** — Animated canvas or CSS particles layered on top.
 *
 * Themes whose draw functions interleave scene+effect (warzone, wasteland, sandstorm)
 * keep scene:'none' — splitting them is a future refactor.
 */
export const THEME_CONFIG: Record<string, ThemeConfig> = {
	// Core
	dark:              { style: 'dark',              scene: 'none',                  effect: 'none' },
	light:             { style: 'light',             scene: 'none',                  effect: 'none' },
	midnight:          { style: 'midnight',          scene: 'none',                  effect: 'stars' },
	forest:            { style: 'forest',            scene: 'forest',                effect: 'none' },
	ocean:             { style: 'ocean',             scene: 'none',                  effect: 'water' },
	desert:            { style: 'desert',            scene: 'none',                  effect: 'cactus' },
	aurora:            { style: 'aurora',             scene: 'none',                  effect: 'stars' },
	cream:             { style: 'cream',             scene: 'none',                  effect: 'none' },
	sky:               { style: 'sky',               scene: 'none',                  effect: 'clouds' },
	lavender:          { style: 'lavender',          scene: 'none',                  effect: 'stars' },
	sage:              { style: 'sage',              scene: 'none',                  effect: 'leaves' },
	peach:             { style: 'peach',             scene: 'none',                  effect: 'fireflies' },

	// Cinematic
	terminator:        { style: 'terminator',        scene: 'none',                  effect: 'warzone' },
	matrix:            { style: 'matrix',            scene: 'none',                  effect: 'digital-rain' },
	replicant:         { style: 'replicant',         scene: 'rooftops',              effect: 'rain' },
	br2049:            { style: 'br2049',            scene: 'none',                  effect: 'wasteland' },
	'br2049-sandstorm': { style: 'br2049-sandstorm', scene: 'none',                  effect: 'sandstorm' },

	// Seasonal
	snow:              { style: 'snow',              scene: 'winter-landscape',      effect: 'snow' },
	spring:            { style: 'spring',            scene: 'none',                  effect: 'sakura' },
	summer:            { style: 'summer',            scene: 'none',                  effect: 'fireflies' },
	autumn:            { style: 'autumn',            scene: 'none',                  effect: 'leaves' },

	// Holiday
	christmas:         { style: 'christmas',         scene: 'winter-landscape-xmas', effect: 'snow' },
	halloween:         { style: 'halloween',         scene: 'none',                  effect: 'flames' },
	newyear:           { style: 'newyear',           scene: 'none',                  effect: 'fireworks' },
	valentine:         { style: 'valentine',         scene: 'none',                  effect: 'hearts' },

	// Fun
	boba:              { style: 'boba',              scene: 'none',                  effect: 'boba' },
	fruitsalad:        { style: 'fruitsalad',        scene: 'none',                  effect: 'fruit' },
	junkfood:          { style: 'junkfood',          scene: 'none',                  effect: 'junkfood' },
	candy:             { style: 'candy',             scene: 'none',                  effect: 'candy' },
	coffee:            { style: 'coffee',            scene: 'none',                  effect: 'coffee' },
	lava:              { style: 'lava',              scene: 'none',                  effect: 'lava' },
}
