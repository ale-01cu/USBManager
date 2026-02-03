<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { Button, Card, CardHeader, CardTitle, Badge, Separator } from "$lib/components/ui";
  import { ScrollArea } from "$lib/components/ui";
  import Timeline from "$lib/components/custom/Timeline.svelte";
  import FileTree from "$lib/components/custom/FileTree.svelte";
  import KpiCard from "$lib/components/custom/KpiCard.svelte";
  import { ArrowLeft, HardDrive, Folder, FileText, Clock, Calendar, Loader2 } from "lucide-svelte";
  
  interface FileNode {
    name: string;
    path: string;
    is_folder: boolean;
    size: number;
    extension?: string;
    children: FileNode[];
    expanded?: boolean;
  }
  
  interface TimelineEvent {
    id: number;
    type: 'CONNECT' | 'DISCONNECT';
    timestamp: string;
    duration?: string;
    file_count?: number;
    folder_count?: number;
  }
  
  interface DeviceScan {
    activity_id: number;
    timestamp: string;
    snapshot_count: number;
    file_count: number;
    folder_count: number;
  }
  
  interface DeviceFilesResponse {
    success: boolean;
    device_id: string;
    activity_id: number;
    snapshots: Array<{
      path: string;
      is_folder: boolean;
      size: number;
    }>;
    stats: {
      total_files: number;
      total_folders: number;
    };
  }
  
  interface DeviceScansResponse {
    success: boolean;
    device_id: string;
    scans: DeviceScan[];
  }
  
  interface DeviceDetails {
    serial_number: string;
    name?: string;
    manufacturer?: string;
    total_capacity?: number;
  }
  
  // Get serial from URL params
  let serial = $derived($page.params.serial);
  
  // State
  let deviceDetails = $state<DeviceDetails | null>(null);
  let scans = $state<DeviceScan[]>([]);
  let selectedActivityId = $state<number>(0);
  let fileTree = $state<FileNode | null>(null);
  let selectedFile = $state<FileNode | null>(null);
  let isLoadingDevice = $state(true);
  let isLoadingFiles = $state(false);
  let totalSessions = $state(0);
  let totalFilesScanned = $state(0);
  let totalFoldersScanned = $state(0);
  
  // Derived timeline events from scans
  let timelineEvents = $derived<TimelineEvent[]>(
    scans.map((scan, index) => ({
      id: scan.activity_id,
      type: 'CONNECT' as const,
      timestamp: scan.timestamp,
      file_count: scan.file_count,
      folder_count: scan.folder_count,
      // Calculate duration from next scan if available
      duration: index < scans.length - 1 
        ? calculateDuration(new Date(scan.timestamp), new Date(scans[index + 1].timestamp))
        : undefined
    }))
  );
  
  function calculateDuration(start: Date, end: Date): string {
    const diffMs = end.getTime() - start.getTime();
    const diffHours = Math.floor(diffMs / 3600000);
    const diffMins = Math.floor((diffMs % 3600000) / 60000);
    
    if (diffHours > 0) {
      return `${diffHours}h ${diffMins}m`;
    }
    return `${diffMins}m`;
  }
  
  function formatBytes(bytes?: number): string {
    if (!bytes) return "Unknown";
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    if (bytes === 0) return "0 B";
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + " " + sizes[i];
  }
  
  function getFileExtension(filename: string): string | undefined {
    const parts = filename.split('.');
    if (parts.length > 1 && !filename.startsWith('.')) {
      return parts[parts.length - 1].toLowerCase();
    }
    return undefined;
  }
  
  function buildFileTree(snapshots: Array<{ path: string; is_folder: boolean; size: number }>): FileNode {
    const root: FileNode = {
      name: 'Root',
      path: '/',
      is_folder: true,
      size: 0,
      children: [],
      expanded: true
    };
    
    for (const snapshot of snapshots) {
      const parts = snapshot.path.split('/').filter(p => p.length > 0);
      let current = root;
      let currentPath = '';
      
      for (let i = 0; i < parts.length; i++) {
        const part = parts[i];
        currentPath += '/' + part;
        const isLast = i === parts.length - 1;
        
        // Check if this node already exists
        let existingNode = current.children.find(child => child.name === part);
        
        if (!existingNode) {
          const newNode: FileNode = {
            name: part,
            path: currentPath,
            is_folder: isLast ? snapshot.is_folder : true,
            size: isLast ? snapshot.size : 0,
            extension: !isLast || snapshot.is_folder ? undefined : getFileExtension(part),
            children: [],
            expanded: false
          };
          current.children.push(newNode);
          current = newNode;
        } else {
          current = existingNode;
        }
      }
    }
    
    // Sort children: folders first, then alphabetically
    sortFileNodes(root);
    
    return root;
  }
  
  function sortFileNodes(node: FileNode) {
    node.children.sort((a, b) => {
      // Folders come before files
      if (a.is_folder && !b.is_folder) return -1;
      if (!a.is_folder && b.is_folder) return 1;
      // Alphabetical within same type
      return a.name.localeCompare(b.name);
    });
    
    // Recursively sort children
    for (const child of node.children) {
      if (child.children.length > 0) {
        sortFileNodes(child);
      }
    }
  }
  
  async function loadDeviceData() {
    try {
      isLoadingDevice = true;
      
      // Load all scans for this device
      const scansResult = await invoke<DeviceScansResponse>("get_device_all_scans", {
        deviceId: serial
      });
      
      if (scansResult.success) {
        scans = scansResult.scans;
        totalSessions = scans.length;
        totalFilesScanned = scans.reduce((sum, scan) => sum + scan.file_count, 0);
        totalFoldersScanned = scans.reduce((sum, scan) => sum + scan.folder_count, 0);
        
        // Select the most recent scan by default
        if (scans.length > 0) {
          selectedActivityId = scans[0].activity_id;
          await loadFilesForActivity(selectedActivityId);
        }
      }
    } catch (error) {
      console.error("Failed to load device data:", error);
    } finally {
      isLoadingDevice = false;
    }
  }
  
  async function loadFilesForActivity(activityId: number) {
    try {
      isLoadingFiles = true;
      
      const result = await invoke<DeviceFilesResponse>("get_device_files", {
        deviceId: serial
      });
      
      if (result.success && result.snapshots.length > 0) {
        fileTree = buildFileTree(result.snapshots);
      } else {
        fileTree = null;
      }
    } catch (error) {
      console.error("Failed to load files:", error);
      fileTree = null;
    } finally {
      isLoadingFiles = false;
    }
  }
  
  function handleTimelineSelect(event: TimelineEvent) {
    selectedActivityId = event.id;
    loadFilesForActivity(event.id);
  }
  
  function handleFileSelect(node: FileNode) {
    selectedFile = node;
  }
  
  function goBack() {
    goto('/devices');
  }
  
  onMount(() => {
    loadDeviceData();
  });
</script>

<div class="h-full flex flex-col">
  <!-- Header -->
  <div class="flex items-center gap-4 mb-6">
    <Button variant="ghost" size="sm" onclick={goBack}>
      <ArrowLeft class="h-4 w-4 mr-2" />
      Back to Device Vault
    </Button>
    
    <Separator orientation="vertical" class="h-6" />
    
    <div class="flex items-center gap-3">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10">
        <HardDrive class="h-5 w-5 text-primary" />
      </div>
      <div>
        <h1 class="text-xl font-bold text-foreground">{deviceDetails?.name || 'Unknown Device'}</h1>
        <p class="text-sm text-muted-foreground">Serial: {serial}</p>
      </div>
    </div>
    
    {#if deviceDetails?.manufacturer}
      <Badge variant="secondary">{deviceDetails.manufacturer}</Badge>
    {/if}
    
    {#if deviceDetails?.total_capacity}
      <Badge variant="outline">{formatBytes(deviceDetails.total_capacity)}</Badge>
    {/if}
  </div>
  
  <!-- KPI Cards -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
    <KpiCard
      title="Total Sessions"
      value={totalSessions}
      icon={Clock}
      subtitle="Connection events tracked"
      iconColor="bg-info/10 text-info"
    />
    
    <KpiCard
      title="Files Scanned"
      value={totalFilesScanned.toLocaleString()}
      icon={FileText}
      subtitle="Total files across all sessions"
      iconColor="bg-success/10 text-success"
    />
    
    <KpiCard
      title="Folders Scanned"
      value={totalFoldersScanned.toLocaleString()}
      icon={Folder}
      subtitle="Total directories indexed"
      iconColor="bg-warning/10 text-warning"
    />
  </div>
  
  <!-- Split Pane Layout -->
  <div class="flex-1 flex border border-border rounded-lg overflow-hidden">
    <!-- Left Panel: Timeline (35%) -->
    <div class="w-[35%] border-r border-border flex flex-col bg-card">
      <CardHeader class="border-b border-border py-4">
        <CardTitle class="text-lg flex items-center gap-2">
          <Calendar class="h-5 w-5 text-muted-foreground" />
          Connection History
        </CardTitle>
      </CardHeader>
      
      <ScrollArea class="flex-1 p-4">
        {#if isLoadingDevice}
          <div class="flex items-center justify-center h-32">
            <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        {:else if timelineEvents.length > 0}
          <Timeline
            events={timelineEvents}
            selectedId={selectedActivityId}
            onSelect={handleTimelineSelect}
          />
        {:else}
          <div class="text-center py-8">
            <Clock class="mx-auto h-12 w-12 text-muted-foreground/50 mb-3" />
            <p class="text-sm text-muted-foreground">No connection history available</p>
          </div>
        {/if}
      </ScrollArea>
    </div>
    
    <!-- Right Panel: File Explorer (65%) -->
    <div class="w-[65%] flex flex-col bg-card">
      <CardHeader class="border-b border-border py-4">
        <CardTitle class="text-lg flex items-center gap-2">
          <Folder class="h-5 w-5 text-muted-foreground" />
          File Explorer
          {#if selectedActivityId > 0}
            <Badge variant="secondary" class="ml-2">Session #{selectedActivityId}</Badge>
          {/if}
        </CardTitle>
      </CardHeader>
      
      <ScrollArea class="flex-1 p-4">
        {#if isLoadingFiles}
          <div class="flex items-center justify-center h-64">
            <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        {:else if fileTree && fileTree.children.length > 0}
          {#each fileTree.children as child (child.path)}
            <FileTree
              node={child}
              onSelect={handleFileSelect}
              selectedPath={selectedFile?.path || ''}
            />
          {/each}
        {:else}
          <div class="flex flex-col items-center justify-center h-64 text-center">
            <Folder class="h-16 w-16 text-muted-foreground/50 mb-4" />
            <h3 class="text-lg font-medium text-foreground">No files available</h3>
            <p class="text-sm text-muted-foreground mt-1 max-w-sm">
              {#if selectedActivityId === 0}
                Select a connection event from the timeline to view its file snapshot
              {:else}
                No files were scanned during this session
              {/if}
            </p>
          </div>
        {/if}
      </ScrollArea>
      
      <!-- File Details Footer -->
      {#if selectedFile}
        <div class="border-t border-border p-4 bg-muted/50">
          <div class="flex items-center gap-4">
            <FileText class="h-5 w-5 text-muted-foreground" />
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate">{selectedFile.name}</p>
              <p class="text-xs text-muted-foreground">{selectedFile.path}</p>
            </div>
            {#if !selectedFile.is_folder}
              <Badge variant="outline">{formatBytes(selectedFile.size)}</Badge>
            {/if}
            {#if selectedFile.extension}
              <Badge variant="secondary">.{selectedFile.extension}</Badge>
            {/if}
            <Badge variant={selectedFile.is_folder ? "default" : "outline"}>
              {selectedFile.is_folder ? 'Folder' : 'File'}
            </Badge>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>
