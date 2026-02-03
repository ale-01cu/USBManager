<script lang="ts">
  import { cn } from '$lib/utils/cn';
  import { Plug, Unplug, Clock } from 'lucide-svelte';
  
  interface TimelineEvent {
    id: number;
    type: 'CONNECT' | 'DISCONNECT';
    timestamp: string;
    duration?: string;
  }
  
  let { 
    events,
    selectedId,
    onSelect
  } = $props<{
    events: TimelineEvent[];
    selectedId?: number;
    onSelect?: (event: TimelineEvent) => void;
  }>();
  
  function formatDate(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    });
  }
  
  function formatTime(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit'
    });
  }
  
  function getRelativeTime(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);
    
    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return formatDate(timestamp);
  }
</script>

<div class="relative">
  <!-- Timeline line -->
  <div class="absolute left-6 top-0 bottom-0 w-px bg-border"></div>
  
  <!-- Events -->
  <div class="space-y-1">
    {#each events as event, i (event.id)}
      {@const isSelected = selectedId === event.id}
      {@const isConnect = event.type === 'CONNECT'}
      
      <button
        class={cn(
          "relative flex items-start gap-4 w-full text-left p-3 rounded-lg transition-colors",
          "hover:bg-accent/50",
          isSelected && "bg-accent"
        )}
        onclick={() => onSelect?.(event)}
      >
        <!-- Icon -->
        <div class={cn(
          "relative z-10 flex h-8 w-8 items-center justify-center rounded-full border-2 flex-shrink-0",
          isConnect 
            ? "bg-success/10 border-success text-success" 
            : "bg-destructive/10 border-destructive text-destructive"
        )}>
          {#if isConnect}
            <Plug class="h-4 w-4" />
          {:else}
            <Unplug class="h-4 w-4" />
          {/if}
        </div>
        
        <!-- Content -->
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class={cn(
              "text-sm font-semibold",
              isConnect ? "text-success" : "text-destructive"
            )}>
              {isConnect ? 'Connected' : 'Disconnected'}
            </span>
            <span class="text-xs text-muted-foreground">
              {getRelativeTime(event.timestamp)}
            </span>
          </div>
          
          <div class="flex items-center gap-3 mt-1 text-xs text-muted-foreground">
            <span class="flex items-center gap-1">
              <Clock class="h-3 w-3" />
              {formatTime(event.timestamp)}
            </span>
            <span>{formatDate(event.timestamp)}</span>
          </div>
          
          {#if event.duration && isConnect}
            <div class="mt-2 text-xs">
              <span class="inline-flex items-center gap-1 px-2 py-1 rounded-full bg-muted">
                <Clock class="h-3 w-3" />
                Duration: {event.duration}
              </span>
            </div>
          {/if}
        </div>
      </button>
    {/each}
  </div>
</div>
