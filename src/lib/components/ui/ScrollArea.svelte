<script lang="ts">
	import { cn } from '$lib/utils/cn';

	interface Props {
		orientation?: 'vertical' | 'horizontal';
		class?: string;
		children?: () => any;
		[key: string]: any;
	}

	let {
		orientation = 'horizontal',
		class: className = '',
		children,
		...rest
	}: Props = $props();

	const baseClasses = 'relative overflow-hidden';
	
	const orientationClasses = {
		vertical: 'h-full w-full',
		horizontal: 'h-full w-full'
	};

	const classes = $derived(cn(baseClasses, orientationClasses[orientation], className));
</script>

<div class={classes} {...rest}>
	<div
		class={cn(
			'overflow-auto scrollbar-thin scrollbar-track-muted scrollbar-thumb-muted-foreground/30 hover:scrollbar-thumb-muted-foreground/50',
			orientation === 'vertical' && 'h-full overflow-x-hidden',
			orientation === 'horizontal' && 'w-full overflow-y-hidden'
		)}
	>
		{@render children?.()}
	</div>
</div>

<style>
	.scrollbar-thin {
		scrollbar-width: thin;
	}

	.scrollbar-thin::-webkit-scrollbar {
		width: 8px;
		height: 8px;
	}

	.scrollbar-thin::-webkit-scrollbar-track {
		background: hsl(var(--muted));
	}

	.scrollbar-thin::-webkit-scrollbar-thumb {
		background: hsl(var(--muted-foreground) / 0.3);
		border-radius: 4px;
	}

	.scrollbar-thin::-webkit-scrollbar-thumb:hover {
		background: hsl(var(--muted-foreground) / 0.5);
	}
</style>
