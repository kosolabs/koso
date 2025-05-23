<script lang="ts">
  import { page } from "$app/state";
  import { headers, parseResponse } from "$lib/api";
  import { auth } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { toast } from "$lib/components/ui/sonner";
  import { deleteUserConnection, redirectToConnectUserFlow } from "$lib/github";
  import { Button } from "$lib/kosui/button";
  import { getDialoguerContext } from "$lib/kosui/dialog";
  import { Link } from "$lib/kosui/link";
  import { CircularProgress } from "$lib/kosui/progress";
  import { ToggleButton, ToggleGroup } from "$lib/kosui/toggle";
  import { CircleX, Moon, Send, Sun, SunMoon, Trash2 } from "lucide-svelte";
  import { userPrefersMode as mode } from "mode-watcher";
  import Section from "./section.svelte";
  import SubSection from "./sub-section.svelte";

  const dialog = getDialoguerContext();

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

  type PluginConnections = {
    githubUserId?: string;
  };

  type Profile = {
    notificationConfigs: NotificationConfig[];
    pluginConnections: PluginConnections;
  };

  async function load(): Promise<Profile> {
    let resp = await fetch("/api/profile", { headers: headers() });
    return await parseResponse(resp);
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
      await parseResponse(resp);
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
      await parseResponse(resp);
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

  async function deleteUserGithubConnection() {
    if (
      !(await dialog.confirm({
        message,
        title: "Delete Github Connection?",
        icon: Trash2,
      }))
    ) {
      return;
    }

    const toastId = toast.loading("Deleting Github connection...");

    try {
      await deleteUserConnection();
      toast.success("Github connection deleted.", { id: toastId });
      profile = load();
    } catch {
      toast.error("Failed to delete Github connection.", { id: toastId });
    }
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
    <ToggleGroup bind:value={mode.current}>
      <ToggleButton value="light">
        <Sun size={16} />
        Light
      </ToggleButton>
      <ToggleButton value="dark">
        <Moon size={16} />
        Dark
      </ToggleButton>
      <ToggleButton value="system">
        <SunMoon size={16} />
        System
      </ToggleButton>
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
  <Section title="Plugins">
    {#await profile}
      <div class="flex place-content-center items-center gap-2">
        <CircularProgress />
        <div>Loading...</div>
      </div>
    {:then profile}
      <SubSection title="Github">
        {@const githubUserId = profile.pluginConnections.githubUserId}
        {#if githubUserId}
          <div class="flex flex-col gap-2">
            <div>
              Your Koso profile is connected to Github user
              <Link href="https://api.github.com/user/{githubUserId}">
                {githubUserId}
              </Link>
            </div>
            <div class="flex flex-wrap gap-2">
              <div class="ml-auto">
                <Button
                  icon={CircleX}
                  variant="filled"
                  onclick={async () => await deleteUserGithubConnection()}
                >
                  Delete Connection
                </Button>
              </div>
            </div>
            <div></div>
          </div>
        {:else}
          Your Koso profile is not connected to Github.
          <Button
            icon={CircleX}
            variant="filled"
            onclick={async () =>
              await redirectToConnectUserFlow(page.url.pathname)}
          >
            Connect to Github
          </Button>
        {/if}
      </SubSection>
    {/await}
  </Section>
</div>
