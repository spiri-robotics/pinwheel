import type { Job } from '$lib/gen'
import { PINWHEEL_VIEWBOX, pinwheelMarkup } from '$lib/components/icons/pinwheelMark'

export type JobStatusKind = 'running' | 'success' | 'failure' | 'canceled'

const STATUS_COLORS: Record<JobStatusKind, string> = {
	running: '#eab308',
	success: '#22c55e',
	failure: '#ef4444',
	canceled: '#6b7280'
}

const DEFAULT_FAVICON = '/logo.svg'

function faviconLink(): HTMLLinkElement {
	let link = document.querySelector<HTMLLinkElement>('link[rel="icon"]')
	if (!link) {
		link = document.createElement('link')
		link.rel = 'icon'
		document.head.appendChild(link)
	}
	return link
}

/** Maps a job to one of the favicon status colors, following the same
 * discrimination as JobStatusIcon (`'success' in job` => completed). A job that
 * is still running while being canceled stays 'running' until it completes. */
export function getJobStatusKind(job: Job | undefined): JobStatusKind | undefined {
	if (!job) return undefined
	if (job.canceled && 'success' in job) return 'canceled'
	if ('success' in job) return job.success ? 'success' : 'failure'
	return 'running'
}

export function setStatusFavicon(status: JobStatusKind): void {
	const color = STATUS_COLORS[status]
	const svg =
		`<svg xmlns="http://www.w3.org/2000/svg" viewBox="${PINWHEEL_VIEWBOX}">` +
		`<g>${pinwheelMarkup()}</g>` +
		'<circle cx="188" cy="188" r="62" fill="#ffffff"/>' +
		`<circle cx="188" cy="188" r="48" fill="${color}"/>` +
		'</svg>'
	faviconLink().href = `data:image/svg+xml,${encodeURIComponent(svg)}`
}

export function resetFavicon(): void {
	faviconLink().href = DEFAULT_FAVICON
}
