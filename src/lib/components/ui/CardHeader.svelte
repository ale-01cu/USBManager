<script lang="ts" module>
	import { cn } from '$lib/utils/cn';
	import type { HTMLAttributes } from 'svelte/elements';

	export interface CardHeaderProps extends HTMLAttributes<HTMLDivElement> {
		class?: string;
	}
</script>

<script lang="ts">
	let { class: className = '', children, ...rest }: CardHeaderProps = $props();
	const classes = $derived(cn('flex flex-col space-y-1.5 p-6', className));
</script>

<div class={classes} {...rest}>
	{@render children?.()}
</div>
