<script lang="ts">
  import { KosoLogo } from "$lib/components/ui/koso-logo";
  import Navbar from "$lib/navbar.svelte";
</script>

<Navbar />

<div class="flex items-center justify-center pt-8">
  <div class="flex w-9/12 max-w-[425px] rounded-md border bg-muted p-4">
    <div class="min-w-16">
      <KosoLogo class="size-16" />
    </div>
    <div class="ml-4">
      <div class="text-md">Welcome to Koso!</div>
      <div class="mt-2 text-xs">
        We're excited that you're excited! We're not available quite yet, but
        stay tuned.
      </div>
    </div>
  </div>
</div>
