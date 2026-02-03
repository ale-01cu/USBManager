<script lang="ts">
	import { cn } from '$lib/utils/cn';

	interface Props {
		class?: string;
		[key: string]: any;
	}

	let { class: className = '', ...rest }: Props = $props();

	const classes = $derived(cn('animate-pulse rounded-md bg-muted', className));
</script>

<div class={classes} {...rest}></div>
