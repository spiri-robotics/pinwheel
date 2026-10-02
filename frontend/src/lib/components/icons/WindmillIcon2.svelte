<script lang="ts">
	import { run } from 'svelte/legacy'

	import { customIcon } from './store'
	import {
		PINWHEEL_BLADE,
		PINWHEEL_BLADE_ROTATIONS,
		PINWHEEL_FLAP,
		PINWHEEL_HUB,
		PINWHEEL_PRIMARY,
		PINWHEEL_SECONDARY,
		PINWHEEL_VIEWBOX,
		PINWHEEL_WHITE_PRIMARY,
		PINWHEEL_WHITE_SECONDARY
	} from './pinwheelMark'

	interface Props {
		white?: boolean
		size?: string
		color?: string | undefined
		spin?: 'slow' | 'medium' | 'fast' | 'veryfast' | undefined
		class?: string
	}

	let {
		white = false,
		size = '24px',
		color = undefined,
		spin = undefined,
		class: classNames = ''
	}: Props = $props()

	function hslToHex(h, s, l) {
		s /= 100
		l /= 100

		let c = (1 - Math.abs(2 * l - 1)) * s
		let x = c * (1 - Math.abs(((h / 60) % 2) - 1))
		let m = l - c / 2
		let r = 0
		let g = 0
		let b = 0

		if (0 <= h && h < 60) {
			r = c
			g = x
			b = 0
		} else if (60 <= h && h < 120) {
			r = x
			g = c
			b = 0
		} else if (120 <= h && h < 180) {
			r = 0
			g = c
			b = x
		} else if (180 <= h && h < 240) {
			r = 0
			g = x
			b = c
		} else if (240 <= h && h < 300) {
			r = x
			g = 0
			b = c
		} else if (300 <= h && h < 360) {
			r = c
			g = 0
			b = x
		}

		let rs = Math.round((r + m) * 255)
			.toString(16)
			.padStart(2, '0')
		let gs = Math.round((g + m) * 255)
			.toString(16)
			.padStart(2, '0')
		let bs = Math.round((b + m) * 255)
			.toString(16)
			.padStart(2, '0')

		return `#${rs}${gs}${bs}`
	}

	function hexToHsl(hex) {
		let r: number = parseInt(hex.slice(1, 3), 16) / 255
		let g: number = parseInt(hex.slice(3, 5), 16) / 255
		let b: number = parseInt(hex.slice(5, 7), 16) / 255

		const max = Math.max(r, g, b),
			min = Math.min(r, g, b)
		let h,
			s,
			l = (max + min) / 2

		if (max === min) {
			h = s = 0 // Achromatic
		} else {
			const d = max - min
			s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
			switch (max) {
				case r:
					h = (g - b) / d + (g < b ? 6 : 0)
					break
				case g:
					h = (b - r) / d + 2
					break
				case b:
					h = (r - g) / d + 4
					break
			}
			h /= 6
		}

		return [h * 360, s * 100, l * 100]
	}

	function reduceSaturation(hex: string, reductionPercent: number) {
		// Convert HEX to HSL

		// Convert the hex to HSL
		let [h, s, l] = hexToHsl(hex)

		// Reduce the saturation by the specified percentage
		l = Math.max(0, l - reductionPercent)

		// Convert back to hex
		return hslToHex(h, s, l)
	}

	let lessSaturatedColor: string | undefined = $state()

	run(() => {
		color ? (lessSaturatedColor = reduceSaturation(color, -16)) : (lessSaturatedColor = undefined)
	})
</script>

{#if customIcon.white || customIcon.normal}
	{#if white}
		<img
			src={customIcon.white}
			alt="Pinwheel Custom icon"
			width={size}
			height={size}
			class={classNames}
		/>
	{:else}
		<img
			src={customIcon.normal}
			alt="Pinwheel Custom icon"
			width={size}
			height={size}
			class={classNames}
		/>
	{/if}
{:else}
	<svg
		class={classNames}
		class:animate-[spin_2s_linear_infinite]={spin === 'veryfast'}
		class:animate-[spin_5s_linear_infinite]={spin === 'fast'}
		class:animate-[spin_15s_linear_infinite]={spin === 'medium'}
		class:animate-[spin_50s_linear_infinite]={spin === 'slow'}
		xmlns="http://www.w3.org/2000/svg"
		width={size}
		height={size}
		viewBox={PINWHEEL_VIEWBOX}
	>
		<!-- Use color or fallback to defaults (white or blue) -->
		{#each PINWHEEL_BLADE_ROTATIONS as rotation}
			<g transform="rotate({rotation} 128 128)">
				<path
					fill={color || (white ? PINWHEEL_WHITE_PRIMARY : PINWHEEL_PRIMARY)}
					d={PINWHEEL_BLADE}
				/>
				<path
					fill={lessSaturatedColor || (white ? PINWHEEL_WHITE_SECONDARY : PINWHEEL_SECONDARY)}
					d={PINWHEEL_FLAP}
				/>
			</g>
		{/each}
		<circle
			cx={PINWHEEL_HUB.cx}
			cy={PINWHEEL_HUB.cy}
			r={PINWHEEL_HUB.r}
			fill={color || (white ? PINWHEEL_WHITE_PRIMARY : PINWHEEL_PRIMARY)}
		/>
	</svg>
{/if}
