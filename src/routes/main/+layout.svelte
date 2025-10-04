<script lang="ts">
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";

  import AppSidebar from "./sidebar.svelte";
  import Header from "./header.svelte";

  let { children } = $props();
</script>

<div class="bg-background text-foreground min-h-screen">
  <Sidebar.Provider>
    <AppSidebar />
    <Sidebar.Inset>
      <Header />
      <main>
        <div class="mx-auto px-8 py-4">
          {@render children()}
        </div>
      </main>
    </Sidebar.Inset>
  </Sidebar.Provider>
</div>
