<script lang="ts">
	import { Alert } from '$lib/components/common'
	import AlignWrapper from '../helpers/AlignWrapper.svelte'
</script>

<AlignWrapper verticalAlignment="center" horizontalAlignment="center">
	<Alert
		title="Deprecated component"
		documentationLink="https://github.com/spiri-robotics/windmill-OSS"
		tooltip="See documentation of the new component:"
		type="error"
	>
		This component is deprecated and has been removed. Please use the Log by Job Id component
		instead.
	</Alert>
</AlignWrapper>
