<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { headers, parse_response } from "$lib/api";
  import { Alert } from "$lib/components/ui/alert";
  import { dialog } from "$lib/kosui/dialog";
  import { CircularProgress } from "$lib/kosui/progress";
  import Navbar from "$lib/navbar.svelte";
  import { CircleCheck, CircleSlash, CircleX } from "lucide-svelte";
  import { onMount } from "svelte";

  let loading = $state(true);

  onMount(async () => {
    const token = page.url.searchParams.get("token");

    if (!token) {
      loading = false;
      await dialog.notice({
        icon: CircleSlash,
        title: "Telegram authorization failed",
        message:
          "Koso failed to authorize sending notifications to Telegram because the token was missing. Return to the User Profile page and try again.",
        acceptText: "Return to User Profile",
      });
      return await goto("/profile");
    }

    let resp = await fetch("/api/notifiers/telegram", {
      method: "POST",
      headers: {
        ...headers(),
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ token }),
    });
    loading = false;

    try {
      await parse_response(resp);
      const goHome = await dialog.confirm({
        icon: CircleCheck,
        title: "Telegram authorized",
        message: "Koso has been authorized to send notifications to Telegram.",
        cancelText: "User Profile",
        acceptText: "Return Home",
      });
      if (goHome) {
        return await goto("/");
      } else {
        return await goto("/profile");
      }
    } catch {
      await dialog.notice({
        icon: CircleX,
        title: "Telegram authorization failed",
        message:
          "Koso failed to authorize sending notifications to Telegram because the token was invalid. Return to the User Profile page and try again.",
        acceptText: "Return to User Profile",
      });
      return await goto("/profile");
    }
  });
</script>

<Navbar />

{#if loading}
  <div class="m-2">
    <Alert>
      <div class="flex items-center gap-2">
        <CircularProgress />
        <div>Koso is authorizing sending notifications to Telegram...</div>
      </div>
    </Alert>
  </div>
{/if}
