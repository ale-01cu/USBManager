<script lang="ts" module>
	import { cn } from '$lib/utils/cn';
	import type { HTMLAttributes } from 'svelte/elements';

	export interface CardTitleProps extends HTMLAttributes<HTMLHeadingElement> {
		class?: string;
	}
</script>

<script lang="ts">
	let { class: className = '', children, ...rest }: CardTitleProps = $props();
	const classes = $derived(cn('text-2xl font-semibold leading-none tracking-tight', className));
</script>

<h3 class={classes} {...rest}>
	{@render children?.()}
</h3>
