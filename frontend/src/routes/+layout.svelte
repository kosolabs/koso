<script lang="ts">
  import { Toaster } from "$lib/components/ui/sonner";
  import { ModeWatcher } from "mode-watcher";
  import "../app.css";
</script>

<ModeWatcher />
<Toaster />
<slot />
