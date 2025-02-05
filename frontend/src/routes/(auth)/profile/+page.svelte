<script lang="ts">
  import { headers, parse_response } from "$lib/api";
  import { auth } from "$lib/auth.svelte";
  import { Button } from "$lib/components/ui/button";
  import { IndeterminateProgress } from "$lib/components/ui/circular-progress";
  import { Toggle } from "$lib/components/ui/toggle";
  import { dialog } from "$lib/kosui/dialog";
  import { Link } from "$lib/kosui/link";
  import Navbar from "$lib/navbar.svelte";
  import { CircleX, Moon, Send, Sun, SunMoon, Trash2 } from "lucide-svelte";
  import { userPrefersMode as mode, setMode } from "mode-watcher";
  import { toast } from "svelte-sonner";
  import Section from "./section.svelte";
  import SubSection from "./sub-section.svelte";

  let profile: Promise<Profile> = $state(load());

  type Base = {
    email: string;
    enabled: boolean;
  };

  type TelegramNotificationConfig = Base & {
    notifier: "telegram";
    settings: {
      chatId: number;
    };
  };

  type NotificationConfig = TelegramNotificationConfig;

  type Profile = {
    notificationConfigs: NotificationConfig[];
  };

  async function load(): Promise<Profile> {
    let resp = await fetch("/api/profile", { headers: headers() });
    return await parse_response(resp);
  }

  async function sendTestTelegramNotification() {
    const toastId = toast.loading("Sending test notification...");

    try {
      let resp = await fetch("/api/notifiers/telegram/test", {
        method: "POST",
        headers: {
          ...headers(),
          "Content-Type": "application/json",
        },
      });
      await parse_response(resp);
      toast.success("Test notification sent successfully.", { id: toastId });
    } catch {
      toast.error("Failed to send test notification.", { id: toastId });
    }
  }

  async function deleteTelegramConfig() {
    if (
      !(await dialog.confirm({
        message,
        title: "Delete Telegram Authorization?",
        icon: Trash2,
      }))
    ) {
      return;
    }

    const toastId = toast.loading("Deleting Telegram authorization...");

    try {
      let resp = await fetch("/api/notifiers/telegram", {
        method: "DELETE",
        headers: {
          ...headers(),
          "Content-Type": "application/json",
        },
      });
      await parse_response(resp);
      toast.success("Telegram authorization deleted.", { id: toastId });
      profile = load();
    } catch {
      toast.error("Failed to delete Telegram authorization.", { id: toastId });
    }
  }

  function getTelegramConfig(
    profile: Profile,
  ): TelegramNotificationConfig | null {
    for (const config of profile.notificationConfigs) {
      if (config.notifier === "telegram") {
        return config;
      }
    }
    return null;
  }
</script>

{#snippet message()}
  Deleting the Telegram authorization will prevent Koso from being able to send
  notifications to Telegram.
{/snippet}

<Navbar>
  {#snippet left()}
    <div>
      <h1 class="ml-2 text-lg">Profile Settings for {auth.user.name}</h1>
    </div>
  {/snippet}
</Navbar>

<div class="flex flex-col p-2">
  <Section title="Theme">
    <Toggle
      variant="outline"
      bind:pressed={() => $mode === "light", () => setMode("light")}
    >
      <Sun />
      Light
    </Toggle>
    <Toggle
      variant="outline"
      bind:pressed={() => $mode === "dark", () => setMode("dark")}
    >
      <Moon />
      Dark
    </Toggle>
    <Toggle
      variant="outline"
      bind:pressed={() => $mode === "system", () => setMode("system")}
    >
      <SunMoon />
      System
    </Toggle>
  </Section>
  <Section title="Notifications">
    {#await profile}
      <IndeterminateProgress /> Loading
    {:then profile}
      <SubSection title="Telegram">
        {@const telegramConfig = getTelegramConfig(profile)}
        {#if telegramConfig}
          <div class="flex flex-col gap-2">
            <div>Koso is authorized to send messages to Telegram.</div>
            <div class="flex flex-wrap gap-2">
              <Button onclick={sendTestTelegramNotification}>
                <Send />
                Send Test Notification
              </Button>
              <div class="ml-auto">
                <Button variant="destructive" onclick={deleteTelegramConfig}>
                  <CircleX />
                  Delete Authorization
                </Button>
              </div>
            </div>
            <div></div>
          </div>
        {:else}
          Koso is not authorized to send messages to Telegram. To authorize
          Koso, start a Telegram Chat with
          <Link href="https://t.me/KosoLabsBot">@KosoLabsBot</Link>.
        {/if}
      </SubSection>
    {/await}
  </Section>
</div>
