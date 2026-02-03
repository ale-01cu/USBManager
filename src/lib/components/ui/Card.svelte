<script lang="ts" module>
	import { cn } from '$lib/utils/cn';
	import type { HTMLAttributes } from 'svelte/elements';

	// Card Root
	interface CardRootProps extends HTMLAttributes<HTMLDivElement> {
		class?: string;
	}

	// Card Header
	interface CardHeaderProps extends HTMLAttributes<HTMLDivElement> {
		class?: string;
	}

	// Card Title
	interface CardTitleProps extends HTMLAttributes<HTMLHeadingElement> {
		class?: string;
	}

	// Card Description
	interface CardDescriptionProps extends HTMLAttributes<HTMLParagraphElement> {
		class?: string;
	}

	// Card Content
	interface CardContentProps extends HTMLAttributes<HTMLDivElement> {
		class?: string;
	}

	// Card Footer
	interface CardFooterProps extends HTMLAttributes<HTMLDivElement> {
		class?: string;
	}

	export { type CardRootProps, type CardHeaderProps, type CardTitleProps, type CardDescriptionProps, type CardContentProps, type CardFooterProps };
</script>

<script lang="ts">
	let { class: className = '', children, ...rest }: CardRootProps = $props();
	const classes = $derived(cn('rounded-lg border bg-card text-card-foreground shadow-sm', className));
</script>

<div class={classes} {...rest}>
	{@render children?.()}
</div>
