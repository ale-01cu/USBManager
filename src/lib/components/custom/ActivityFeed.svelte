<script lang="ts">
  import { cn } from '$lib/utils/cn';
  import { Card, CardContent, CardHeader, CardTitle, Badge, ScrollArea } from '$lib/components/ui';
  import { Plug, Unplug, Usb, Activity } from 'lucide-svelte';

  interface ActivityEvent {
    id: string | number;
    deviceName: string;
    timestamp: string;
    eventType: 'CONNECT' | 'DISCONNECT' | 'SCAN' | 'ERROR';
    details?: string;
  }

  interface Props {
    events: ActivityEvent[];
    maxItems?: number;
    title?: string;
    class?: string;
  }

  let {
    events,
    maxItems = 50,
    title = 'Live Activity Feed',
    class: className = ''
  }: Props = $props();

  const displayedEvents = $derived(events.slice(0, maxItems));

  const eventConfig = {
    CONNECT: {
      icon: Plug,
      badgeVariant: 'success' as const,
      label: 'Connected',
      color: 'text-success'
    },
    DISCONNECT: {
      icon: Unplug,
      badgeVariant: 'destructive' as const,
      label: 'Disconnected',
      color: 'text-destructive'
    },
    SCAN: {
      icon: Usb,
      badgeVariant: 'info' as const,
      label: 'Scanned',
      color: 'text-info'
    },
    ERROR: {
      icon: Activity,
      badgeVariant: 'destructive' as const,
      label: 'Error',
      color: 'text-destructive'
    }
  };

  function formatTimestamp(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffSecs = Math.floor(diffMs / 1000);
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);

    if (diffSecs < 10) return 'Just now';
    if (diffSecs < 60) return `${diffSecs}s ago`;
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  }
</script>

<Card class={cn('overflow-hidden', className)}>
  <CardHeader class="pb-3">
    <div class="flex items-center justify-between">
      <CardTitle class="text-base font-semibold flex items-center gap-2">
        {title}
        <span class="relative flex h-2.5 w-2.5">
          <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-success opacity-75"></span>
          <span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-success"></span>
        </span>
      </CardTitle>
      <span class="text-xs text-muted-foreground">
        {displayedEvents.length} events
      </span>
    </div>
  </CardHeader>
  <CardContent class="p-0">
    <ScrollArea class="h-[300px]">
      <div class="space-y-1 p-4 pt-0">
        {#if displayedEvents.length === 0}
          <div class="flex flex-col items-center justify-center py-8 text-muted-foreground">
            <Activity class="h-8 w-8 mb-2 opacity-50" />
            <p class="text-sm">No activity yet</p>
            <p class="text-xs">Connect a device to see events</p>
          </div>
        {:else}
          {#each displayedEvents as event, index (event.id)}
            {@const config = eventConfig[event.eventType]}
            {@const Icon = config.icon}
            
            <div
              class={cn(
                'flex items-start gap-3 p-3 rounded-lg transition-colors',
                'hover:bg-accent/50',
                index !== displayedEvents.length - 1 && 'border-b border-border/50'
              )}
            >
              <div class={cn(
                'flex h-8 w-8 items-center justify-center rounded-full flex-shrink-0',
                'bg-muted'
              )}>
                <Icon class={cn('h-4 w-4', config.color)} />
              </div>
              
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 justify-between">
                  <p class="text-sm font-medium truncate">
                    {event.deviceName}
                  </p>
                  <Badge variant={config.badgeVariant} class="text-xs flex-shrink-0">
                    {config.label}
                  </Badge>
                </div>
                
                <div class="flex items-center gap-2 mt-1">
                  <span class="text-xs text-muted-foreground">
                    {formatTimestamp(event.timestamp)}
                  </span>
                  <span class="text-xs text-muted-foreground">
                    {new Date(event.timestamp).toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' })}
                  </span>
                </div>
                
                {#if event.details}
                  <p class="text-xs text-muted-foreground mt-1 truncate">
                    {event.details}
                  </p>
                {/if}
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </ScrollArea>
  </CardContent>
</Card>
