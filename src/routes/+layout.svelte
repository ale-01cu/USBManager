<script lang="ts">
  import { page } from '$app/stores';
  import Sidebar from '$lib/components/custom/Sidebar.svelte';
  import '../app.css';
  
  let currentRoute = $derived($page.url.pathname);
</script>

<div class="min-h-screen bg-background">
  <Sidebar {currentRoute} />
  
  <main class="ml-64 min-h-screen">
    <div class="p-8">
      <slot />
    </div>
  </main>
</div>
