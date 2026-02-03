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
  "w-64 h-full",
  "bg-card border-r border-border",
  "flex flex-col flex-shrink-0",
  className
)}>
  <!-- Navigation -->
  <nav class="flex-1 space-y-1 p-4 pt-6">
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
  
</aside>
