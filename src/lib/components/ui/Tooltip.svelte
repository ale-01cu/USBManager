<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import type { Snippet } from 'svelte';

	interface Props {
		content: string | Snippet;
		position?: 'top' | 'bottom' | 'left' | 'right';
		align?: 'start' | 'center' | 'end';
		sideOffset?: number;
		class?: string;
		children: Snippet;
	}

	let {
		content,
		position = 'top',
		align = 'center',
		sideOffset = 4,
		class: className = '',
		children
	}: Props = $props();

	let isVisible = $state(false);
	let triggerElement: HTMLElement | null = $state(null);

	const baseClasses =
		'z-50 overflow-hidden rounded-md border bg-popover px-3 py-1.5 text-sm text-popover-foreground shadow-md';

	const positionClasses = {
		top: 'bottom-full mb-1',
		bottom: 'top-full mt-1',
		left: 'right-full mr-1',
		right: 'left-full ml-1'
	};

	const alignClasses = {
		start: position === 'left' || position === 'right' ? 'top-0' : 'left-0',
		center: 'left-1/2 -translate-x-1/2',
		end: position === 'left' || position === 'right' ? 'bottom-0' : 'right-0'
	};

	function handleMouseEnter() {
		isVisible = true;
	}

	function handleMouseLeave() {
		isVisible = false;
	}

	function handleFocus() {
		isVisible = true;
	}

	function handleBlur() {
		isVisible = false;
	}
</script>

<div
	class="relative inline-flex"
	bind:this={triggerElement}
	onmouseenter={handleMouseEnter}
	onmouseleave={handleMouseLeave}
	onfocus={handleFocus}
	onblur={handleBlur}
	role="tooltip"
>
	{@render children()}

	{#if isVisible}
		<div
			class={cn(
				'absolute whitespace-nowrap pointer-events-none',
				positionClasses[position],
				alignClasses[align],
				baseClasses,
				className
			)}
			style="{position === 'top' || position === 'bottom'
				? `margin-${position === 'top' ? 'bottom' : 'top'}: ${sideOffset}px;`
				: `margin-${position === 'left' ? 'right' : 'left'}: ${sideOffset}px;`}"
		>
			{#if typeof content === 'string'}
				{content}
			{:else}
				{@render content()}
			{/if}
		</div>
	{/if}
</div>
