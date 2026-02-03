<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { Button, Input } from "$lib/components/ui";
  import DeviceTable from "$lib/components/custom/DeviceTable.svelte";
  import { Database, Search, RefreshCw, Loader2 } from "lucide-svelte";
  
  interface Device {
    serial_number: string;
    vendor_id: number;
    product_id: number;
    name?: string;
    manufacturer?: string;
    total_capacity?: number;
    updated_at?: string;
  }
  
  let devices = $state<Device[]>([]);
  let filteredDevices = $state<Device[]>([]);
  let searchQuery = $state("");
  let isLoading = $state(true);
  
  function formatBytes(bytes?: number): string {
    if (!bytes) return "Unknown";
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    if (bytes === 0) return "0 B";
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + " " + sizes[i];
  }
  
  function formatDate(dateStr?: string): string {
    if (!dateStr) return "Never";
    return new Date(dateStr).toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  }
  
  async function loadDevices() {
    try {
      isLoading = true;
      const result = await invoke<{ success: boolean; devices: Device[] }>(
        "get_registered_devices"
      );
      if (result.success) {
        devices = result.devices;
        filterDevices();
      }
    } catch (error) {
      console.error("Failed to load devices:", error);
    } finally {
      isLoading = false;
    }
  }
  
  function filterDevices() {
    if (!searchQuery.trim()) {
      filteredDevices = devices;
      return;
    }
    
    const query = searchQuery.toLowerCase();
    filteredDevices = devices.filter(
      (d) =>
        d.name?.toLowerCase().includes(query) ||
        d.manufacturer?.toLowerCase().includes(query) ||
        d.serial_number.toLowerCase().includes(query) ||
        d.vendor_id.toString().includes(query) ||
        d.product_id.toString().includes(query)
    );
  }
  
  function handleDeviceClick(device: Device) {
    goto(`/devices/${encodeURIComponent(device.serial_number)}`);
  }
  
  onMount(() => {
    loadDevices();
    
    listen("usb-connected", () => {
      loadDevices();
    });
    
    listen("usb-disconnected", () => {
      loadDevices();
    });
  });
  
  $effect(() => {
    filterDevices();
  });
</script>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
    <div>
      <h1 class="text-3xl font-bold text-foreground">Device Vault</h1>
      <p class="text-muted-foreground mt-1">
        Browse and audit all tracked USB devices
      </p>
    </div>
    
    <Button onclick={loadDevices} variant="outline" disabled={isLoading}>
      {#if isLoading}
        <Loader2 class="mr-2 h-4 w-4 animate-spin" />
        Loading...
      {:else}
        <RefreshCw class="mr-2 h-4 w-4" />
        Refresh
      {/if}
    </Button>
  </div>
  
  <!-- Stats Bar -->
  <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
    <div class="bg-card border border-border rounded-lg p-4">
      <div class="flex items-center gap-2 text-muted-foreground text-sm">
        <Database class="h-4 w-4" />
        Total Devices
      </div>
      <p class="text-2xl font-bold text-foreground mt-1">{devices.length}</p>
    </div>
    
    <div class="bg-card border border-border rounded-lg p-4">
      <div class="flex items-center gap-2 text-muted-foreground text-sm">
        <Database class="h-4 w-4" />
        With Name
      </div>
      <p class="text-2xl font-bold text-foreground mt-1">
        {devices.filter(d => d.name).length}
      </p>
    </div>
    
    <div class="bg-card border border-border rounded-lg p-4">
      <div class="flex items-center gap-2 text-muted-foreground text-sm">
        <Database class="h-4 w-4" />
        Known Capacity
      </div>
      <p class="text-2xl font-bold text-foreground mt-1">
        {devices.filter(d => d.total_capacity).length}
      </p>
    </div>
    
    <div class="bg-card border border-border rounded-lg p-4">
      <div class="flex items-center gap-2 text-muted-foreground text-sm">
        <Database class="h-4 w-4" />
        This Week
      </div>
      <p class="text-2xl font-bold text-foreground mt-1">
        {devices.filter(d => {
          if (!d.updated_at) return false;
          const date = new Date(d.updated_at);
          const weekAgo = new Date();
          weekAgo.setDate(weekAgo.getDate() - 7);
          return date > weekAgo;
        }).length}
      </p>
    </div>
  </div>
  
  <!-- Search -->
  <div class="relative">
    <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
    <Input
      type="text"
      placeholder="Search devices by name, manufacturer, serial number, or IDs..."
      class="pl-10"
      bind:value={searchQuery}
    />
  </div>
  
  <!-- Devices Table -->
  <div class="bg-card border border-border rounded-lg overflow-hidden">
    <DeviceTable
      devices={filteredDevices.map(d => ({
        serial_number: d.serial_number,
        manufacturer: d.manufacturer || "Unknown",
        name: d.name || "Unknown Device",
        capacity: formatBytes(d.total_capacity),
        last_seen: formatDate(d.updated_at)
      }))}
      onDeviceClick={(device) => handleDeviceClick(device as any)}
    />
    
    {#if filteredDevices.length === 0 && !isLoading}
      <div class="text-center py-16">
        <Database class="mx-auto h-16 w-16 text-muted-foreground/50 mb-4" />
        <h3 class="text-lg font-medium text-foreground">No devices found</h3>
        <p class="text-muted-foreground mt-1">
          {searchQuery ? "Try adjusting your search query" : "Connect a USB device to start tracking"}
        </p>
      </div>
    {/if}
  </div>
</div>
