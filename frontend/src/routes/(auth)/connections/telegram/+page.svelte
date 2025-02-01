<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { headers, parse_response } from "$lib/api";
  import { Alert } from "$lib/components/ui/alert";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { IndeterminateProgress } from "$lib/components/ui/circular-progress";
  import Navbar from "$lib/navbar.svelte";
  import { CircleCheck, CircleX } from "lucide-svelte";

  const resp = load();
  type Settings = {
    chatId: number;
  };

  async function load(): Promise<Settings> {
    const token = page.url.searchParams.get("token");

    if (!token) {
      throw new Error("Token is missing");
    }

    let resp = await fetch("/api/notifiers/telegram", {
      method: "POST",
      headers: {
        ...headers(),
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ token }),
    });

    return await parse_response(resp);
  }
</script>

<Navbar />

{#snippet alert(success: boolean)}
  <AlertDialog.Root open={true}>
    <AlertDialog.AlertDialogContent>
      <AlertDialog.AlertDialogHeader>
        <AlertDialog.AlertDialogTitle class="flex items-center gap-1">
          {#if success}
            <CircleCheck size={24} class="text-primary" />
          {:else}
            <CircleX size={24} class="text-destructive" />
          {/if}
          <div>Telegram Authorization</div>
        </AlertDialog.AlertDialogTitle>
        <AlertDialog.AlertDialogDescription class="flex flex-col gap-2">
          {#if success}
            Koso has been authorized to send notifications to Telegram.
          {:else}
            <div>
              Koso failed to authorize sending notifications to Telegram because
              the token was invalid. Steps to connect to Telegram:
            </div>
            <ol>
              <li>1. Open Telegram</li>
              <li>2. Start a Chat with @KosoLabsBot</li>
              <li>3. Send the /token command</li>
              <li>4. Click the link returned by the bot</li>
            </ol>
          {/if}
        </AlertDialog.AlertDialogDescription>
      </AlertDialog.AlertDialogHeader>
      <AlertDialog.AlertDialogFooter>
        <AlertDialog.AlertDialogCancel onclick={() => goto("/")}>
          Return Home
        </AlertDialog.AlertDialogCancel>
        <AlertDialog.AlertDialogAction onclick={() => goto("/profile")}>
          User Profile
        </AlertDialog.AlertDialogAction>
      </AlertDialog.AlertDialogFooter>
    </AlertDialog.AlertDialogContent>
  </AlertDialog.Root>
{/snippet}

{#await resp}
  <div class="m-2">
    <Alert>
      <div class="flex items-center gap-2">
        <IndeterminateProgress />
        <div>Koso is authorizing sending notifications to Telegram...</div>
      </div>
    </Alert>
  </div>
{:then}
  {@render alert(true)}
{:catch}
  {@render alert(false)}
{/await}
