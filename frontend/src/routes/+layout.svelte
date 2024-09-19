<script lang="ts">
  import { Toaster } from "$lib/ui/toaster";
  import { ModeWatcher } from "mode-watcher";
  import "../app.css";
</script>

<ModeWatcher />
<Toaster />
<slot />
