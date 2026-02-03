<script lang="ts">
  import { cn } from '$lib/utils/cn';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui';
  import { HardDrive, FolderOpen } from 'lucide-svelte';

  interface Device {
    serial_number: string;
    vendor_id: number;
    product_id: number;
    name?: string;
    manufacturer?: string;
    total_capacity?: number;
    last_seen?: string;
  }

  interface Props {
    devices: Device[];
    onDeviceClick?: (device: Device) => void;
    class?: string;
  }

  let {
    devices,
    onDeviceClick,
    class: className = ''
  }: Props = $props();

  function formatBytes(bytes: number | undefined): string {
    if (bytes === undefined) return 'Unknown';
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    if (bytes === 0) return '0 Bytes';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
  }

  function formatSerial(serial: string): string {
    if (serial.length <= 16) return serial;
    return serial.substring(0, 8) + '...' + serial.substring(serial.length - 4);
  }

  function formatLastSeen(dateStr: string | undefined): string {
    if (!dateStr) return 'Never';
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  }

  function getDisplayName(device: Device): string {
    return device.name || device.manufacturer || `Device ${device.serial_number.substring(0, 8)}`;
  }

  function getManufacturer(device: Device): string {
    return device.manufacturer || `VID: ${device.vendor_id.toString(16).padStart(4, '0').toUpperCase()}`;
  }
</script>

<Card class={cn('overflow-hidden', className)}>
  <CardHeader class="pb-4">
    <CardTitle class="text-base font-semibold flex items-center gap-2">
      <HardDrive class="h-5 w-5" />
      Device Vault
      <span class="text-sm font-normal text-muted-foreground ml-auto">
        {devices.length} device{devices.length !== 1 ? 's' : ''}
      </span>
    </CardTitle>
  </CardHeader>
  <CardContent class="p-0">
    {#if devices.length === 0}
      <div class="flex flex-col items-center justify-center py-12 px-4 text-center">
        <div class="flex h-12 w-12 items-center justify-center rounded-full bg-muted mb-4">
          <FolderOpen class="h-6 w-6 text-muted-foreground" />
        </div>
        <h3 class="text-sm font-semibold text-foreground mb-1">No devices yet</h3>
        <p class="text-xs text-muted-foreground max-w-[200px]">
          Connect a USB device to start tracking it in your vault
        </p>
      </div>
    {:else}
      <div class="overflow-x-auto">
        <table class="w-full">
          <thead>
            <tr class="border-b border-border bg-muted/50">
              <th class="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                Manufacturer
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                Product Name
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                Serial
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                Capacity
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                Last Seen
              </th>
            </tr>
          </thead>
          <tbody class="divide-y divide-border">
            {#each devices as device (device.serial_number)}
              <tr
                class={cn(
                  'transition-colors',
                  onDeviceClick && 'cursor-pointer hover:bg-accent/50'
                )}
                onclick={() => onDeviceClick?.(device)}
                role={onDeviceClick ? 'button' : undefined}
                tabindex={onDeviceClick ? 0 : undefined}
                onkeydown={(e) => {
                  if (onDeviceClick && (e.key === 'Enter' || e.key === ' ')) {
                    e.preventDefault();
                    onDeviceClick(device);
                  }
                }}
              >
                <td class="px-4 py-3 text-sm">
                  <span class="font-medium text-foreground">
                    {getManufacturer(device)}
                  </span>
                </td>
                <td class="px-4 py-3 text-sm">
                  <span class="text-foreground">
                    {getDisplayName(device)}
                  </span>
                </td>
                <td class="px-4 py-3 text-sm">
                  <code class="text-xs bg-muted px-1.5 py-0.5 rounded">
                    {formatSerial(device.serial_number)}
                  </code>
                </td>
                <td class="px-4 py-3 text-sm">
                  <span class={cn(
                    'text-foreground',
                    !device.total_capacity && 'text-muted-foreground italic'
                  )}>
                    {formatBytes(device.total_capacity)}
                  </span>
                </td>
                <td class="px-4 py-3 text-sm">
                  <span class="text-muted-foreground">
                    {formatLastSeen(device.last_seen)}
                  </span>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </CardContent>
</Card>
