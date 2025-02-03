<script lang="ts">
  import { headers, parse_response } from "$lib/api";
  import { auth } from "$lib/auth.svelte";
  import * as Accordion from "$lib/components/ui/accordion";
  import { Button } from "$lib/components/ui/button";
  import { IndeterminateProgress } from "$lib/components/ui/circular-progress";
  import { Toggle } from "$lib/components/ui/toggle";
  import Navbar from "$lib/navbar.svelte";
  import { CircleX, Moon, Send, Sun, SunMoon } from "lucide-svelte";
  import { userPrefersMode as mode, setMode } from "mode-watcher";
  import { toast } from "svelte-sonner";
  import Section from "./section.svelte";
  import SubSection from "./sub-section.svelte";
  import { dialog } from "$lib/kosui/dialog";

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
    const ok = await dialog.confirm("Wow, great!");

    console.log(`Confirmed: ${ok}`);
    if (!ok) {
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
          <Accordion.Root type="single">
            <Accordion.Item value="item-1">
              <Accordion.Trigger>
                Koso is not authorized to send messages to Telegram. Expand for
                instructions to enable.
              </Accordion.Trigger>
              <Accordion.Content>
                <ol class="ml-4">
                  <li>1. Open Telegram</li>
                  <li>2. Start a Chat with @KosoLabsBot</li>
                  <li>3. Send the /token command</li>
                  <li>4. Click the link returned by the bot</li>
                </ol>
              </Accordion.Content>
            </Accordion.Item>
          </Accordion.Root>
        {/if}
      </SubSection>
    {/await}
  </Section>
</div>
