<script lang="ts">
  import { page } from '$app/stores';
  import Sidebar from '$lib/components/custom/Sidebar.svelte';
  import TitleBar from '$lib/components/custom/TitleBar.svelte';
  import '../app.css';
  
  let currentRoute = $derived($page.url.pathname);
</script>

<div class="min-h-screen bg-background flex flex-col">
  <!-- Custom Title Bar - Full Width at Top -->
  <TitleBar />
  
  <!-- Content Area: Sidebar + Main Content below TitleBar -->
  <div class="pt-14 flex flex-1 overflow-hidden">
    <!-- Sidebar -->
    <Sidebar {currentRoute} />
    
    <!-- Main Content -->
    <main class="flex-1 overflow-y-auto">
      <div class="p-8">
        <slot />
      </div>
    </main>
  </div>
</div>
