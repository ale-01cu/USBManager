<script lang="ts">
  import { cn } from "$lib/utils/cn";

  let {
    value = $bindable(),
    placeholder = "",
    type = "text",
    disabled = false,
    class: className = "",
    ...props
  } = $props<{
    value?: string;
    placeholder?: string;
    type?: string;
    disabled?: boolean;
    class?: string;
  }>();
</script>

<input
  {type}
  {placeholder}
  {disabled}
  bind:value
  class={cn(
    "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background",
    "file:border-0 file:bg-transparent file:text-sm file:font-medium",
    "placeholder:text-muted-foreground",
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
    "disabled:cursor-not-allowed disabled:opacity-50",
    className
  )}
  {...props}
/>
