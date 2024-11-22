<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import KosoLogo from "$lib/components/ui/koso-logo/koso-logo.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let name = $state("");
  let greetMsg = $state("");

  async function greet(event: Event) {
    event.preventDefault();
    greetMsg = await invoke("greet", { name });
  }
</script>

<main class="mt-auto flex flex-col items-center gap-16 p-20">
  <h1 class="text-4xl">Welcome to Koso Zero</h1>

  <KosoLogo class="w-40" />

  <div class="flex gap-2">
    <Input bind:value={name} placeholder="Enter a name..." />
    <Button onclick={greet}>Greet</Button>
  </div>
  <p class="text-2xl">{greetMsg}</p>
</main>

<style>
</style>
