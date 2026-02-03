<script lang="ts">
  import { cn } from '$lib/utils/cn';
  import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '$lib/components/ui';
  import { TrendingUp, TrendingDown, type Icon as LucideIcon } from 'lucide-svelte';
  import type { Component } from 'svelte';

  interface Props {
    title: string;
    value: string | number;
    icon: Component;
    trend?: 'up' | 'down' | 'neutral';
    trendValue?: string | number;
    subtitle?: string;
    iconColor?: string;
    class?: string;
  }

  let {
    title,
    value,
    icon: Icon,
    trend,
    trendValue,
    subtitle,
    iconColor = 'bg-primary/10 text-primary',
    class: className = ''
  }: Props = $props();

  const trendConfig = {
    up: { icon: TrendingUp, color: 'text-success', label: 'Increase' },
    down: { icon: TrendingDown, color: 'text-destructive', label: 'Decrease' },
    neutral: { icon: null, color: 'text-muted-foreground', label: 'No change' }
  };

  const currentTrend = $derived(trend ? trendConfig[trend] : null);
</script>

<Card class={cn('overflow-hidden', className)}>
  <CardHeader class="pb-2">
    <div class="flex items-center justify-between">
      <CardTitle class="text-sm font-medium text-muted-foreground">
        {title}
      </CardTitle>
      <div class={cn('flex h-8 w-8 items-center justify-center rounded-full', iconColor)}>
        <Icon class="h-4 w-4" />
      </div>
    </div>
  </CardHeader>
  <CardContent>
    <div class="flex flex-col gap-1">
      <div class="text-2xl font-bold tracking-tight">
        {value}
      </div>
      
      {#if trend && trendValue !== undefined && currentTrend}
        <div class="flex items-center gap-1.5 text-xs">
          <currentTrend.icon class={cn('h-3.5 w-3.5', currentTrend.color)} />
          <span class={cn('font-medium', currentTrend.color)}>
            {trendValue}
          </span>
          <span class="text-muted-foreground">
            vs last period
          </span>
        </div>
      {/if}
      
      {#if subtitle}
        <CardDescription class="text-xs mt-1">
          {subtitle}
        </CardDescription>
      {/if}
    </div>
  </CardContent>
</Card>
