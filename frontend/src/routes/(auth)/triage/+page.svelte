<script lang="ts">
  import { getAuthContext } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { getProjectContext } from "$lib/dag-table";

  //   const auth = getAuthContext();
  //   const project = getProjectContext();
</script>

<div class="flex h-dvh flex-col">
  <div class="grow-0">
    <Navbar breadcrumbs={["Triage"]}>
      {#snippet left()}
        <div>
          <h1 class="ml-2 text-lg">Triage</h1>
        </div>
      {/snippet}
    </Navbar>
  </div>
</div>
