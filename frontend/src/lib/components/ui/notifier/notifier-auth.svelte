<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { headers, parseResponse } from "$lib/api";
  import { getAuthContext } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { CircleCheck, CircleSlash, CircleX } from "@lucide/svelte";
  import {
    Alert,
    CircularProgress,
    getDialoguerContext,
    toTitleCase,
  } from "kosui";
  import { onMount } from "svelte";

  export type Notifier = "discord" | "slack" | "telegram" | "teams";

  export type NotifierAuthProps = {
    notifier: Notifier;
  };

  let { notifier }: NotifierAuthProps = $props();

  const dialog = getDialoguerContext();
  const auth = getAuthContext();

  let loading = $state(true);

  async function authorize(notifier: Notifier) {
    const name = toTitleCase(notifier);
    const token = page.url.searchParams.get("token");

    if (!token) {
      loading = false;
      await dialog.notice({
        icon: CircleSlash,
        title: `${name} authorization failed`,
        message: `Koso failed to authorize sending notifications to ${name} because the token was missing. Return to the User Profile page and try again.`,
        acceptText: "Return to User Profile",
      });
      return await goto("/profile");
    }

    const resp = await fetch(`/api/notifiers/${notifier}`, {
      method: "POST",
      headers: {
        ...headers(auth),
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ token }),
    });
    loading = false;

    try {
      await parseResponse(auth, resp);
      const goHome = await dialog.confirm({
        icon: CircleCheck,
        title: `${name} authorized`,
        message: `Koso has been authorized to send notifications to ${name}.`,
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
        title: `${name} authorization failed`,
        message: `Koso failed to authorize sending notifications to ${name} because the token was invalid. Return to the User Profile page and try again.`,
        acceptText: "Return to User Profile",
      });
      return await goto("/profile");
    }
  }

  onMount(() => authorize(notifier));
</script>

<Navbar />

{#if loading}
  <div class="m-2">
    <Alert>
      <div class="flex items-center gap-2">
        <CircularProgress />
        <div>
          Koso is authorizing sending notifications to {toTitleCase(
            notifier,
          )}...
        </div>
      </div>
    </Alert>
  </div>
{/if}
