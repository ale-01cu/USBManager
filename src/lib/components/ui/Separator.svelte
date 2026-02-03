<script lang="ts">
	import { cn } from '$lib/utils/cn';

	interface Props {
		orientation?: 'horizontal' | 'vertical';
		decorative?: boolean;
		class?: string;
		[key: string]: any;
	}

	let {
		orientation = 'horizontal',
		decorative = true,
		class: className = '',
		...rest
	}: Props = $props();

	const baseClasses = 'shrink-0 bg-border';

	const orientationClasses = {
		horizontal: 'h-[1px] w-full',
		vertical: 'h-full w-[1px]'
	};

	const classes = $derived(cn(baseClasses, orientationClasses[orientation], className));
</script>

{#if decorative}
	<div role="none" class={classes} {...rest}></div>
{:else}
	<div
		role="separator"
		aria-orientation={orientation}
		class={classes}
		{...rest}
	></div>
{/if}
