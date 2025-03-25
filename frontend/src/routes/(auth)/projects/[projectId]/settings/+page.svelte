<script lang="ts">
  import { CREDENTIAL_KEY } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { toast } from "$lib/components/ui/sonner";
  import { Button } from "$lib/kosui/button";

  async function clearLocalState() {
    [...Array(localStorage.length).keys()]
      .map((i) => localStorage.key(i))
      .forEach((key) => {
        if (key && key !== CREDENTIAL_KEY) {
          console.debug(`Removing local storage item '${key}'`);
          localStorage.removeItem(key);
        }
      });
    sessionStorage.clear();
    const dbs = await window.indexedDB.databases();
    for (const db of dbs) {
      if (db.name) {
        console.debug(`Deleting IndexedDB: '${db.name}'`);
        window.indexedDB.deleteDatabase(db.name);
      }
    }

    toast.info("Local state cleared!");
  }
</script>

<Navbar />

<div class="p-4">
  <Button onclick={clearLocalState}>Clear local state</Button>
</div>
