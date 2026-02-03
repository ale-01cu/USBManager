<script lang="ts" module>
	import { cn } from '$lib/utils/cn';
	import type { HTMLAttributes } from 'svelte/elements';

	export interface CardContentProps extends HTMLAttributes<HTMLDivElement> {
		class?: string;
	}
</script>

<script lang="ts">
	let { class: className = '', children, ...rest }: CardContentProps = $props();
	const classes = $derived(cn('p-6 pt-0', className));
</script>

<div class={classes} {...rest}>
	{@render children?.()}
</div>
