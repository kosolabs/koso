<script lang="ts">
  import { headers, parse_response } from "$lib/api";
  import { auth } from "$lib/auth.svelte";
  import { toast } from "$lib/components/ui/sonner";
  import { Button } from "$lib/kosui/button";
  import { dialog } from "$lib/kosui/dialog";
  import { Link } from "$lib/kosui/link";
  import { CircularProgress } from "$lib/kosui/progress";
  import { ToggleButton, ToggleGroup } from "$lib/kosui/toggle";
  import Navbar from "$lib/navbar.svelte";
  import { CircleX, Moon, Send, Sun, SunMoon, Trash2 } from "lucide-svelte";
  import { userPrefersMode as mode } from "mode-watcher";
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

<div class="flex flex-col gap-4 p-2">
  <Section title="Theme">
    <ToggleGroup bind:value={$mode}>
      {#snippet children(toggleGroup)}
        <ToggleButton {toggleGroup} value="light">
          <Sun size={16} />
          Light
        </ToggleButton>
        <ToggleButton {toggleGroup} value="dark">
          <Moon size={16} />
          Dark
        </ToggleButton>
        <ToggleButton {toggleGroup} value="system">
          <SunMoon size={16} />
          System
        </ToggleButton>
      {/snippet}
    </ToggleGroup>
  </Section>
  <Section title="Notifications">
    {#await profile}
      <div class="flex place-content-center items-center gap-2">
        <CircularProgress />
        <div>Loading...</div>
      </div>
    {:then profile}
      <SubSection title="Telegram">
        {@const telegramConfig = getTelegramConfig(profile)}
        {#if telegramConfig}
          <div class="flex flex-col gap-2">
            <div>Koso is authorized to send messages to Telegram.</div>
            <div class="flex flex-wrap gap-2">
              <Button icon={Send} onclick={sendTestTelegramNotification}>
                Send Test Notification
              </Button>
              <div class="ml-auto">
                <Button
                  icon={CircleX}
                  variant="filled"
                  onclick={deleteTelegramConfig}
                >
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
