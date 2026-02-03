<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { cn } from '$lib/utils/cn';
  import { HardDrive, Minus, Square, X } from 'lucide-svelte';
  
  let isMaximized = $state(false);
  
  async function minimize() {
    await invoke('minimize_window');
  }
  
  async function toggleMaximize() {
    await invoke('toggle_maximize_window');
    isMaximized = !isMaximized;
  }
  
  async function close() {
    await invoke('close_window');
  }
  
  async function handleDoubleClick() {
    await toggleMaximize();
  }
</script>

<header class={cn(
  "fixed top-0 right-0 left-0 z-50 h-14",
  "bg-card/80 backdrop-blur-md",
  "border-b border-border",
  "flex items-center justify-between",
  "select-none"
)}>
  <!-- Left: Logo & Title -->
  <div class="flex items-center gap-3 px-4" data-tauri-drag-region ondblclick={handleDoubleClick}>
    <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-primary text-primary-foreground shadow-sm">
      <HardDrive class="h-5 w-5" />
    </div>
    <div>
      <h1 class="font-semibold text-sm text-foreground leading-tight">USB Manager</h1>
      <p class="text-[10px] text-muted-foreground leading-tight">Forensic Tracker</p>
    </div>
  </div>
  
  <!-- Center: Drag region -->
  <div 
    class="flex-1 h-full" 
    data-tauri-drag-region 
    ondblclick={handleDoubleClick}
  ></div>
  
  <!-- Right: Window Controls -->
  <div class="flex items-center h-full">
    <button
      class="h-full px-4 flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
      onclick={minimize}
      title="Minimize"
    >
      <Minus class="h-4 w-4" />
    </button>
    
    <button
      class="h-full px-4 flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
      onclick={toggleMaximize}
      title={isMaximized ? "Restore" : "Maximize"}
    >
      <Square class="h-4 w-4" />
    </button>
    
    <button
      class="h-full px-4 flex items-center justify-center text-muted-foreground hover:text-destructive hover:bg-destructive/20 transition-colors"
      onclick={close}
      title="Close"
    >
      <X class="h-4 w-4" />
    </button>
  </div>
</header>

<style>
  /* Prevent text selection on the title bar */
  header {
    -webkit-user-select: none;
    user-select: none;
  }
</style>
