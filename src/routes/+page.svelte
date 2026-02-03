<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import { Button, Badge } from "$lib/components/ui";
  import KpiCard from "$lib/components/custom/KpiCard.svelte";
  import ActivityFeed from "$lib/components/custom/ActivityFeed.svelte";
  import {
    HardDrive,
    Activity,
    Usb,
    Loader2
  } from "lucide-svelte";

  interface Device {
    serial_number: string;
    vendor_id: number;
    product_id: number;
    name?: string;
    manufacturer?: string;
    total_capacity?: number;
  }

  interface ActivityLog {
    id: number;
    device_id: string;
    event_type: "CONNECT" | "DISCONNECT";
    timestamp: string;
  }

  interface UsbDevice {
    id: string;
    vendor_id: number;
    product_id: number;
    product_name?: string;
    manufacturer_name?: string;
    serial_number?: string;
  }

  interface FeedEvent {
    id: number;
    deviceName: string;
    eventType: "CONNECT" | "DISCONNECT" | "SCAN";
    timestamp: string;
  }

  let devices = $state<Device[]>([]);
  let activities = $state<ActivityLog[]>([]);
  let connectedDevices = $state<UsbDevice[]>([]);
  let realTimeCopies = $state<any[]>([]);

  let isLoading = $state(true);
  let isFetching = false;
  let intervalId: any;

  let lastEvent = $state<string | null>(null);
  let totalDevices = $derived(devices.length);

  let todayEvents = $derived(
    activities.filter(a => {
      const date = new Date(a.timestamp);
      const today = new Date();
      return date.toDateString() === today.toDateString();
    }).length
  );

  let feedEvents = $derived<FeedEvent[]>(
    activities.slice(0, 10).map(a => {
      const device = devices.find(d => d.serial_number === a.device_id);
      return {
        id: a.id,
        deviceName: device?.name || device?.manufacturer || "Unknown Device",
        eventType: a.event_type,
        timestamp: a.timestamp
      };
    })
  );

  async function loadData() {
    if (isFetching) return;

    try {
      isFetching = true;
      // No poner isLoading = true aquí cada vez para evitar parpadeos en el refresh automático
      if (devices.length === 0 && activities.length === 0) {
        isLoading = true;
      }

      console.log("Fetching dashboard data...");
      const [devicesResult, historyResult, connectedResult] = await Promise.all([
        invoke<{ success: boolean; devices: Device[] }>("get_registered_devices").catch(e => {
          console.error("Error fetching registered devices:", e);
          return { success: false, devices: [] };
        }),
        invoke<{ success: boolean; history: ActivityLog[] }>("get_device_history", { limit: 50 }).catch(e => {
          console.error("Error fetching history:", e);
          return { success: false, history: [] };
        }),
        invoke<UsbDevice[]>("get_connected_devices").catch(e => {
          console.error("Error fetching connected devices:", e);
          return [];
        })
      ]);

      console.log("Results:", { devicesResult, historyResult, connectedResult });

      if (devicesResult.success) {
        devices = devicesResult.devices;
      }

      if (historyResult.success) {
        activities = historyResult.history;
      }

      connectedDevices = connectedResult;

      if (activities.length > 0) {
        const lastActivity = activities[0];
        const device = devices.find(d => d.serial_number === lastActivity.device_id);
        lastEvent = device?.name || device?.manufacturer || "Unknown Device";
      }
    } catch (error) {
      console.error("Failed to load dashboard data:", error);
    } finally {
      isLoading = false;
      isFetching = false;
    }
  }

  onMount(() => {
    loadData();

    const unlistenConnected = listen<UsbDevice>("usb-connected", () => loadData());
    const unlistenDisconnected = listen<UsbDevice>("usb-disconnected", () => loadData());

    const unlistenCopy = listen("file-copy-detected", (event: any) => {
      realTimeCopies = [event.payload, ...realTimeCopies].slice(0, 10);
      loadData();
    });

    intervalId = setInterval(loadData, 5000);

    return () => {
      clearInterval(intervalId);
      unlistenConnected.then(f => f());
      unlistenDisconnected.then(f => f());
      unlistenCopy.then(f => f());
    };
  });

  onDestroy(() => {
    if (intervalId) clearInterval(intervalId);
  });
</script>

<div class="space-y-8">
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold text-foreground">Dashboard</h1>
      <p class="text-muted-foreground mt-1">USB Device Monitoring & Forensic Tracking</p>
    </div>
    <Button onclick={loadData} variant="outline" disabled={isLoading}>
      {#if isLoading}
        <Loader2 class="mr-2 h-4 w-4 animate-spin" />
        Loading...
      {:else}
        Refresh Data
      {/if}
    </Button>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
    <KpiCard
      title="Devices Tracked"
      value={totalDevices}
      icon={HardDrive}
      subtitle="Total unique devices in database"
    />
    <KpiCard
      title="Activity Today"
      value={todayEvents}
      icon={Activity}
      subtitle="Connection events today"
      trend={todayEvents > 0 ? "up" : undefined}
      trendValue={todayEvents > 0 ? "+" + todayEvents : undefined}
    />
    <KpiCard
      title="Last Detection"
      value={lastEvent || "None"}
      icon={Usb}
      subtitle={lastEvent ? "Recently detected device" : "No recent activity"}
    />
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <div class="lg:col-span-2">
      <div class="bg-card border border-border rounded-lg p-6">
        <div class="flex items-center justify-between mb-6">
          <h2 class="text-xl font-semibold text-card-foreground">Live Activity Feed</h2>
          <div class="flex items-center gap-2">
            <span class="relative flex h-3 w-3">
              <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-success opacity-75"></span>
              <span class="relative inline-flex rounded-full h-3 w-3 bg-success"></span>
            </span>
            <span class="text-sm text-muted-foreground">Live</span>
          </div>
        </div>
        <ActivityFeed events={feedEvents} maxItems={15} />
      </div>
    </div>

    <div class="lg:col-span-1">
      <div class="bg-card border border-border rounded-lg p-6">
        <h2 class="text-xl font-semibold text-card-foreground mb-6">
          Currently Connected
        </h2>
        {#if connectedDevices.length === 0}
          <div class="text-center py-8 text-muted-foreground">
            <Usb class="mx-auto h-12 w-12 mb-3 opacity-50" />
            <p>No devices connected</p>
            <p class="text-sm mt-1">Connect a USB device to see it here</p>
          </div>
        {:else}
          <div class="space-y-3">
            {#each connectedDevices as device}
              <div class="flex items-center gap-3 p-3 rounded-lg bg-success/10 border border-success/20">
                <div class="h-10 w-10 rounded-full bg-success/20 flex items-center justify-center">
                  <Usb class="h-5 w-5 text-success" />
                </div>
                <div class="flex-1 min-w-0">
                  <p class="font-medium text-foreground truncate">
                    {device.product_name || device.manufacturer_name || "Unknown Device"}
                  </p>
                  <p class="text-xs text-muted-foreground">
                    VID: {device.vendor_id.toString(16).padStart(4, '0').toUpperCase()} |
                    PID: {device.product_id.toString(16).padStart(4, '0').toUpperCase()}
                  </p>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class="mt-6 bg-card border border-warning/20 rounded-lg p-4">
        <h3 class="text-sm font-bold text-warning flex items-center gap-2 mb-3">
          <Activity class="h-4 w-4" />
          MONITOREO DE COPIAS (Venta en curso)
        </h3>
        {#if realTimeCopies.length === 0}
          <p class="text-xs text-muted-foreground italic">Esperando transferencia de archivos...</p>
        {:else}
          <div class="space-y-2">
            {#each realTimeCopies as file}
              <div class="flex justify-between items-center text-xs p-2 bg-accent/30 rounded">
                <span class="truncate max-w-[200px]">{file.file_name}</span>
                <Badge variant="outline">{(file.file_size / 1024 / 1024).toFixed(2)} MB</Badge>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
