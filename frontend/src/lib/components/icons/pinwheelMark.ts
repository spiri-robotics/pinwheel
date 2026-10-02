// The Pinwheel mark: four blades, each a dark triangle with a lighter curled flap, around a
// hub that is left empty so the mark works on any background. Drawn on a 256 grid centred on
// (128, 128); the blades reach 20..236, and the viewBox crops to a 12-unit margin.

export const PINWHEEL_VIEWBOX = '14 14 228 228'

export const PINWHEEL_PRIMARY = '#2F6FEB'
export const PINWHEEL_SECONDARY = '#A9C6FB'
export const PINWHEEL_WHITE_PRIMARY = '#ffffff'
export const PINWHEEL_WHITE_SECONDARY = '#cccccc'

export const PINWHEEL_BLADE_ROTATIONS = [0, 90, 180, 270]
export const PINWHEEL_BLADE = 'M128 116 L128 20 L236 20 L136.49 119.51 Z'
export const PINWHEEL_FLAP = 'M136.49 119.51 L236 20 Q206 104 139.5 124.5 Z'
export const PINWHEEL_HUB = { cx: 128, cy: 128, r: 6 }

/** The mark's shapes as SVG markup, for contexts that need a string (favicons, data URIs). */
export function pinwheelMarkup(
	primary: string = PINWHEEL_PRIMARY,
	secondary: string = PINWHEEL_SECONDARY
): string {
	const blades = PINWHEEL_BLADE_ROTATIONS.map(
		(r) =>
			`<g transform="rotate(${r} 128 128)">` +
			`<path fill="${primary}" d="${PINWHEEL_BLADE}"/>` +
			`<path fill="${secondary}" d="${PINWHEEL_FLAP}"/>` +
			'</g>'
	).join('')
	const { cx, cy, r } = PINWHEEL_HUB
	return `${blades}<circle cx="${cx}" cy="${cy}" r="${r}" fill="${primary}"/>`
}
