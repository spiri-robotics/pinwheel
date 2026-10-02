export type Changelog = {
	label: string
	href: string
	date: string
}

const changelogs: Changelog[] = [
	{
		label: 'AWS Bedrock support for Pinwheel AI',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2025-11-19'
	},

	{
		label: 'Dynamic select for flows',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2025-08-08'
	},

	{
		label: 'MQTT triggers',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2025-03-11'
	},
	{
		label: 'SQS triggers',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2025-02-18'
	},
	{
		label: 'Teams workspace integration',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2025-02-13'
	},
	{
		label: 'Mocked API files',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2025-01-27'
	},
	{
		label: 'Select Python version',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2025-01-24'
	},
	{
		label: 'Postgres triggers',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2025-01-24'
	},
	{
		label: 'Oracle support',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2025-01-15'
	},
	{
		label: 'NATS triggers',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2025-01-15'
	},
	{
		label: 'Workspace color',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2025-01-10'
	},
	{
		label: 'Interactive Slack approval steps',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-12-20'
	},
	{
		label: 'C#',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-12-13'
	},
	{
		label: 'App custom URL',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-12-05'
	},
	{
		label: 'Full text search on jobs and logs',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-12-05'
	},
	{
		label: 'Force dark/light theme in apps',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-11-28'
	},
	{
		label: 'Kafka triggers',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-11-18'
	},
	{
		label: 'Critical channels in UI',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-11-15'
	},
	{
		label: 'Support for Mistral and Anthropic AI models',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-11-14'
	},
	{
		label: 'Websocket triggers',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-11-06'
	},
	{
		label: 'Autoscaling',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-10-28'
	},
	{
		label: 'File download helper',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-10-12'
	},
	{
		label: 'Queue metric alerts',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-10-10'
	},
	{
		label: 'Deno 2.0',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-10-10'
	},
	{
		label: 'Move components inside containers with ctrl+click',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-10-09'
	},
	{
		label: 'Support workers to run natively on Windows',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-10-03'
	},
	{
		label: 'Quick access menu for faster component insertion',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-10-03'
	},
	{
		label: 'Custom HTTP routes',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-09-23'
	},
	{
		label: 'Set/Get progress from code',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-09-18'
	},
	{
		label: 'Directly edit flow YAML',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-09-02'
	},
	{
		label: 'Critical alert channels',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-09-01'
	},
	{
		label: 'See service logs directly in Pinwheel',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-09-01'
	},
	{
		label: 'Vim support for Monaco/webeditor',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-08-28'
	},
	{
		label: 'Hide / Show App Editor Panels',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-08-26'
	},
	{
		label: 'Email triggers',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-08-06'
	},
	{
		label: 'Continue on disapproval/timeout',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-08-14'
	},
	{
		label: 'Nativets runtime supports npm packages and relative imports',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-07-29'
	},
	{
		label: 'App bar as components',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-07-29'
	},
	{
		label: 'TypeScript Bun scripts are automatically pre-bundled',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-07-26'
	},
	{
		label: 'Dynamic select',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-07-22'
	},
	{
		label: 'Flow Status Viewer improvements',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-07-14'
	},
	{
		label: 'Navbar component',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-07-05'
	},
	{
		label: 'Flow versioning',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-07-04'
	},
	{
		label: 'OneOf inputs',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-06-17'
	},
	{
		label: 'Tracking relative imports to avoid dependency hell',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-06-10'
	},
	{
		label: 'Pinwheel Customer Portal',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-06-04'
	},
	{
		label: 'Secondary storage',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-05-31'
	},
	{
		label: 'Allow User Resources in Apps with a toggle',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-05-27'
	},
	{
		label: 'Pinwheel AI now supports GPT-4o',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-05-27'
	},
	{
		label: 'Concurrency limit observability',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-05-15'
	},
	{
		label: 'Full height components',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-05-15'
	},
	{
		label: 'PHP Support',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-05-15'
	},
	{
		label: 'nativets/REST supports the full wmill API',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-05-13'
	},
	{
		label: 'Workers metrics',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-05-10'
	},
	{
		label: 'CLI and Git Sync major improvements',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-04-28'
	},
	{
		label: 'AgGrid Infinite Table',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-04-24'
	},
	{
		label: 'Jobs labels',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-04-24'
	},
	{
		label: 'AgGrid Actions',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-04-12'
	},
	{
		label: 'Continue on error with error as step`s return',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-04-02'
	},
	{
		label: 'While loops',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-04-02'
	},
	{
		label: 'Approval steps improvements',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-03-27'
	},
	{
		label: 'Map Support in Result Renderer',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-03-27'
	},
	{
		label: 'Markdown support in descriptions',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-03-27'
	},
	{
		label: 'Custom flow states',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-03-26'
	},
	{
		label: 'Custom contextual variables',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-03-23'
	},
	{
		label: 'Large log disk and Distributed storage compaction',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-03-23'
	},
	{
		label: 'Rename Workspace (Self-Host only)',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-03-22'
	},
	{
		label: 'Configurable available languages',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-03-13'
	},
	{
		label: 'Workflow as Code',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-03-04'
	},
	{
		label: 'Pin Database in SQL Scripts',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-02-27'
	},
	{
		label: 'Custom workspace secret encryption',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-02-15'
	},
	{
		label: 'Flow & Metadata Copilot',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-02-15'
	},
	{
		label: 'Ag Charts',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-01-24'
	},
	{
		label: 'Database studio',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-01-24'
	},
	{
		label: 'Rich results render',
		href: 'https://github.com/spiri-robotics/windmill-OSS/releases',
		date: '2024-01-23'
	}
]

export { changelogs }
// Single owner of the "which changelogs are new" localStorage key — every
// menu surfacing changelogs must read/stamp through these, or two surfaces
// with independent state would fight over the same key.
const LAST_OPENED_KEY = 'changelogsLastOpened'

export function readRecentChangelogs(): { recent: Changelog[]; hasNew: boolean } {
	const lastOpened = localStorage.getItem(LAST_OPENED_KEY)
	const recent = lastOpened
		? changelogs.filter((changelog) => changelog.date > lastOpened)
		: changelogs.slice(0, 3)
	const hasNew =
		lastOpened != null && recent.length > 0 && lastOpened !== new Date().toISOString().split('T')[0]
	return { recent, hasNew }
}

export function markChangelogsOpened(): void {
	localStorage.setItem(LAST_OPENED_KEY, new Date().toISOString().split('T')[0])
}
