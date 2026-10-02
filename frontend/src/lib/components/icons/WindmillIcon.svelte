<script lang="ts">
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
		height?: string
		width?: string
		white?: boolean
		spin?: 'slow' | 'medium' | 'fast' | 'veryfast' | undefined
		class?: string
		size?: number
	}

	let {
		height: heightProp = '24px',
		width: widthProp = '24px',
		white = false,
		spin = undefined,
		class: classNames = '',
		size = undefined
	}: Props = $props()

	let width = $derived(size ? `${size}px` : widthProp)
	let height = $derived(size ? `${size}px` : heightProp)
	let primary = $derived(white ? PINWHEEL_WHITE_PRIMARY : PINWHEEL_PRIMARY)
	let secondary = $derived(white ? PINWHEEL_WHITE_SECONDARY : PINWHEEL_SECONDARY)
</script>

{#if customIcon.white || customIcon.normal}
	{#if white}
		<img src={customIcon.white} alt="Pinwheel Custom icon" {width} {height} class={classNames} />
	{:else}
		<img src={customIcon.normal} alt="Pinwheel Custom icon" {width} {height} class={classNames} />
	{/if}
{:else}
	<svg
		class={classNames}
		class:animate-[spin_2s_linear_infinite]={spin === 'veryfast'}
		class:animate-[spin_5s_linear_infinite]={spin === 'fast'}
		class:animate-[spin_15s_linear_infinite]={spin === 'medium'}
		class:animate-[spin_50s_linear_infinite]={spin === 'slow'}
		xmlns="http://www.w3.org/2000/svg"
		{width}
		{height}
		viewBox={PINWHEEL_VIEWBOX}
	>
		{#each PINWHEEL_BLADE_ROTATIONS as rotation}
			<g transform="rotate({rotation} 128 128)">
				<path fill={primary} d={PINWHEEL_BLADE} />
				<path fill={secondary} d={PINWHEEL_FLAP} />
			</g>
		{/each}
		<circle cx={PINWHEEL_HUB.cx} cy={PINWHEEL_HUB.cy} r={PINWHEEL_HUB.r} fill={primary} />
	</svg>
{/if}
