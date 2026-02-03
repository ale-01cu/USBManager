<script lang="ts">
  import { cn } from '$lib/utils/cn';
  import { 
    Folder, 
    FolderOpen, 
    File, 
    FileText, 
    FileImage, 
    FileVideo, 
    FileAudio, 
    FileCode,
    FileArchive,
    ChevronRight,
    ChevronDown
  } from 'lucide-svelte';
  
  interface FileNode {
    name: string;
    path: string;
    is_folder: boolean;
    size: number;
    extension?: string;
    children: FileNode[];
    expanded?: boolean;
  }
  
  let { 
    node,
    level = 0,
    onSelect,
    selectedPath = ''
  } = $props<{
    node: FileNode;
    level?: number;
    onSelect?: (node: FileNode) => void;
    selectedPath?: string;
  }>();
  
  let isExpanded = $state(node.expanded ?? false);
  let isSelected = $derived(selectedPath === node.path);
  
  function toggleExpand() {
    if (node.is_folder) {
      isExpanded = !isExpanded;
    }
  }
  
  function handleClick() {
    if (node.is_folder) {
      toggleExpand();
    }
    onSelect?.(node);
  }
  
  function getFileIcon() {
    if (node.is_folder) {
      return isExpanded ? FolderOpen : Folder;
    }
    
    const ext = node.extension?.toLowerCase();
    
    // Images
    if (['jpg', 'jpeg', 'png', 'gif', 'bmp', 'svg', 'webp', 'ico'].includes(ext || '')) {
      return FileImage;
    }
    
    // Videos
    if (['mp4', 'avi', 'mov', 'mkv', 'wmv', 'flv', 'webm'].includes(ext || '')) {
      return FileVideo;
    }
    
    // Audio
    if (['mp3', 'wav', 'flac', 'aac', 'ogg', 'wma', 'm4a'].includes(ext || '')) {
      return FileAudio;
    }
    
    // Code
    if (['js', 'ts', 'jsx', 'tsx', 'html', 'css', 'py', 'java', 'cpp', 'c', 'h', 'json', 'xml'].includes(ext || '')) {
      return FileCode;
    }
    
    // Archives
    if (['zip', 'rar', '7z', 'tar', 'gz', 'bz2'].includes(ext || '')) {
      return FileArchive;
    }
    
    // Documents
    if (['pdf', 'doc', 'docx', 'txt', 'rtf', 'odt'].includes(ext || '')) {
      return FileText;
    }
    
    return File;
  }
  
  function formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(2)} ${sizes[i]}`;
  }
  
  const Icon = getFileIcon();
</script>

<div class="select-none">
  <div
    class={cn(
      "flex items-center gap-2 py-1.5 px-2 rounded-md cursor-pointer transition-colors",
      "hover:bg-accent/50",
      isSelected && "bg-accent text-accent-foreground",
      level > 0 && "ml-4"
    )}
    style="padding-left: {level * 16 + 8}px"
    onclick={handleClick}
    onkeydown={(e) => e.key === 'Enter' && handleClick()}
    role="button"
    tabindex="0"
    aria-expanded={node.is_folder ? isExpanded : undefined}
    aria-selected={isSelected}
  >
    <!-- Expand/Collapse chevron for folders -->
    {#if node.is_folder && node.children.length > 0}
      <span class="text-muted-foreground">
        {#if isExpanded}
          <ChevronDown class="h-4 w-4" />
        {:else}
          <ChevronRight class="h-4 w-4" />
        {/if}
      </span>
    {:else}
      <span class="w-4"></span>
    {/if}
    
    <!-- File/Folder Icon -->
    <span class={cn(
      "flex-shrink-0",
      node.is_folder ? "text-warning" : "text-muted-foreground"
    )}>
      <Icon class="h-5 w-5" />
    </span>
    
    <!-- File Name -->
    <span class="flex-1 truncate text-sm">
      {node.name}
    </span>
    
    <!-- File Size (for files only) -->
    {#if !node.is_folder && node.size > 0}
      <span class="text-xs text-muted-foreground tabular-nums">
        {formatSize(node.size)}
      </span>
    {/if}
  </div>
  
  <!-- Children -->
  {#if node.is_folder && isExpanded && node.children.length > 0}
    <div class="tree-children" role="group">
      {#each node.children as child (child.path)}
        <svelte:self
          node={child}
          level={level + 1}
          {onSelect}
          {selectedPath}
        />
      {/each}
    </div>
  {/if}
</div>

<style>
  .tree-children {
    animation: slideDown 0.2s ease-out;
  }
  
  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-5px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
