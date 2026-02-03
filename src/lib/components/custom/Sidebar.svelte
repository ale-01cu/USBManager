<script lang="ts">
  import { cn } from '$lib/utils/cn';
  import { 
    LayoutDashboard, 
    Database, 
    HardDrive, 
    Settings,
    type Icon as IconType
  } from 'lucide-svelte';
  
  let { 
    currentRoute = '/',
    class: className = ''
  } = $props<{
    currentRoute?: string;
    class?: string;
  }>();
  
  interface NavItem {
    label: string;
    href: string;
    icon: typeof IconType;
    exact?: boolean;
  }
  
  const navItems: NavItem[] = [
    { label: 'Dashboard', href: '/', icon: LayoutDashboard, exact: true },
    { label: 'Device Vault', href: '/devices', icon: Database },
    { label: 'System', href: '/system', icon: HardDrive },
  ];
  
  function isActive(href: string, exact?: boolean): boolean {
    if (exact) return currentRoute === href;
    return currentRoute.startsWith(href);
  }
</script>

<aside class={cn(
  "fixed left-0 top-0 z-40 h-screen w-64",
  "bg-card border-r border-border",
  "flex flex-col",
  className
)}>
  <!-- Logo -->
  <div class="flex h-16 items-center border-b border-border px-6">
    <div class="flex items-center gap-3">
      <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-primary text-primary-foreground">
        <HardDrive class="h-6 w-6" />
      </div>
      <div>
        <h1 class="font-semibold text-lg text-foreground">USB Manager</h1>
        <p class="text-xs text-muted-foreground">Forensic Tracker</p>
      </div>
    </div>
  </div>
  
  <!-- Navigation -->
  <nav class="flex-1 space-y-1 p-4">
    {#each navItems as item}
      <a
        href={item.href}
        class={cn(
          "flex items-center gap-3 rounded-lg px-3 py-2.5",
          "text-sm font-medium transition-colors",
          "hover:bg-accent hover:text-accent-foreground",
          isActive(item.href, item.exact) 
            ? "bg-primary/10 text-primary border-l-2 border-primary" 
            : "text-muted-foreground"
        )}
      >
        <item.icon class="h-5 w-5" />
        {item.label}
      </a>
    {/each}
  </nav>
  
  <!-- Footer -->
  <div class="border-t border-border p-4">
    <div class="flex items-center gap-3 rounded-lg bg-muted/50 px-3 py-2">
      <div class="h-8 w-8 rounded-full bg-primary/20 flex items-center justify-center">
        <Settings class="h-4 w-4 text-primary" />
      </div>
      <div class="flex-1 min-w-0">
        <p class="text-sm font-medium text-foreground truncate">USB Manager</p>
        <p class="text-xs text-muted-foreground">v0.1.0</p>
      </div>
    </div>
  </div>
</aside>
