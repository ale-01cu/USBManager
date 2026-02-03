<script lang="ts" module>
	import { cn } from '$lib/utils/cn';
	import type { HTMLAttributes } from 'svelte/elements';

	export interface CardDescriptionProps extends HTMLAttributes<HTMLParagraphElement> {
		class?: string;
	}
</script>

<script lang="ts">
	let { class: className = '', children, ...rest }: CardDescriptionProps = $props();
	const classes = $derived(cn('text-sm text-muted-foreground', className));
</script>

<p class={classes} {...rest}>
	{@render children?.()}
</p>
