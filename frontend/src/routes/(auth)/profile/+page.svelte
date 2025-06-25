<script lang="ts">
  import { page } from "$app/state";
  import { headers, parseResponse } from "$lib/api";
  import { getAuthContext } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { toast } from "$lib/components/ui/sonner";
  import { deleteUserConnection, redirectToConnectUserFlow } from "$lib/github";
  import { Button } from "$lib/kosui/button";
  import { getDialoguerContext } from "$lib/kosui/dialog";
  import { Input } from "$lib/kosui/input";
  import { Link } from "$lib/kosui/link";
  import { CircularProgress } from "$lib/kosui/progress";
  import { ToggleButton, ToggleGroup } from "$lib/kosui/toggle";
  import {
    CircleX,
    Crown,
    Github,
    Moon,
    Send,
    Sun,
    SunMoon,
    Trash2,
    X,
  } from "@lucide/svelte";
  import { userPrefersMode as mode } from "mode-watcher";
  import Section from "./section.svelte";
  import SubSection from "./sub-section.svelte";

  const dialog = getDialoguerContext();

  let auth = getAuthContext();
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

  type Subscriptions = {
    ownedSubscription?: Subscription;
    status: SubscriptionStatus;
  };

  type Subscription = {
    status: SubscriptionStatus;
    seats: number;
    endTime: string;
    memberEmails: string[];
  };

  type SubscriptionStatus = "None" | "Active" | "Expired";

  type Profile = {
    notificationConfigs: NotificationConfig[];
    pluginConnections: PluginConnections;
    subscriptions: Subscriptions;
  };

  async function load(): Promise<Profile> {
    let resp = await fetch("/api/profile", { headers: headers(auth) });
    return await parseResponse(auth, resp);
  }

  async function sendTestTelegramNotification() {
    const toastId = toast.loading("Sending test notification...");

    try {
      let resp = await fetch("/api/notifiers/telegram/test", {
        method: "POST",
        headers: {
          ...headers(auth),
          "Content-Type": "application/json",
        },
      });
      await parseResponse(auth, resp);
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
          ...headers(auth),
          "Content-Type": "application/json",
        },
      });
      await parseResponse(auth, resp);
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
      await deleteUserConnection(auth);
      toast.success("Github connection deleted.", { id: toastId });
      profile = load();
    } catch {
      toast.error("Failed to delete Github connection.", { id: toastId });
    }
  }

  async function createCheckoutSession() {
    const req: { cancelUrl: string; successUrl: string } = {
      successUrl: `${location.origin}/profile`,
      cancelUrl: `${location.origin}/profile`,
    };

    const response = await fetch(
      `/api/billing/stripe/create-checkout-session`,
      {
        method: "POST",
        headers: {
          ...headers(auth),
          "Content-Type": "application/json",
        },
        body: JSON.stringify(req),
      },
    );
    let res: { redirectUrl: string } = await parseResponse(auth, response);
    console.log("Redirecting to stripe checkout", res);
    window.location.assign(res.redirectUrl);
  }

  async function createPortalSession() {
    const req: { returnUrl: string } = {
      returnUrl: `${location.origin}/profile`,
    };

    const response = await fetch(`/api/billing/stripe/create-portal-session`, {
      method: "POST",
      headers: {
        ...headers(auth),
        "Content-Type": "application/json",
      },
      body: JSON.stringify(req),
    });
    let res: { redirectUrl: string } = await parseResponse(auth, response);
    console.log("Redirecting to stripe portal", res);
    window.location.assign(res.redirectUrl);
  }

  let memberInput: string = $state("");

  async function addMember(member: string) {
    const loadedProfile = await profile;
    const subscription = loadedProfile.subscriptions.ownedSubscription;
    if (!subscription) {
      toast.error("No subscription found. Reload the page and try again");
      return;
    }
    await setMembers([...subscription.memberEmails, member]);
  }

  async function removeMember(member: string) {
    const loadedProfile = await profile;
    const subscription = loadedProfile.subscriptions.ownedSubscription;
    if (!subscription) {
      toast.error("No subscription found. Reload the page and try again");
      return;
    }

    await setMembers(subscription.memberEmails.filter((m) => m != member));
  }

  async function setMembers(members: string[]) {
    const req: { members: string[] } = { members };

    const toastId = toast.loading("Updating subscription members...");

    const response = await fetch(`/api/billing/subscriptions`, {
      method: "PUT",
      headers: {
        ...headers(auth),
        "Content-Type": "application/json",
      },
      body: JSON.stringify(req),
    });
    try {
      await parseResponse(auth, response);

      toast.success("Subscription members updated.", { id: toastId });
      profile = load();
    } catch (e) {
      toast.error("Failed to update subscription members.", { id: toastId });
      throw e;
    }
  }

  async function addAndClear() {
    if (memberInput === "") {
      return;
    }
    const member = memberInput.trim();
    memberInput = "";
    await addMember(member);
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
          <div class="flex flex-col gap-2">
            <div>Your Koso profile is not connected to Github.</div>
            <div class="flex flex-wrap gap-2">
              <Button
                icon={Github}
                onclick={async () =>
                  await redirectToConnectUserFlow(auth, page.url.pathname)}
              >
                Connect to Github
              </Button>
            </div>
            <div></div>
          </div>
        {/if}
      </SubSection>
    {/await}
  </Section>

  <Section title="Subscriptions">
    <SubSection title="Subscription">
      {#await profile}
        <div class="flex place-content-center items-center gap-2">
          <CircularProgress />
          <div>Loading...</div>
        </div>
      {:then profile}
        {@const subs = profile.subscriptions}
        {@const sub = subs.ownedSubscription}

        <div class="flex flex-col gap-2">
          <div>
            {#if subs.status === "None"}
              You do not have an active subscription.
            {:else if subs.status === "Expired"}
              Your subscription expired.
            {:else if subs.status === "Active"}
              You have a premium subscription.
            {:else}
              Something went wrong. Invalid subscription status "{subs.status}".
              Let us know!
            {/if}
          </div>

          {#if sub && sub.status === "Active"}
            <div class="flex flex-wrap gap-2">
              <div class="ml-auto">
                <Button
                  icon={Crown}
                  variant="filled"
                  onclick={async () => await createPortalSession()}
                >
                  Manage
                </Button>
              </div>
            </div>
          {:else if sub || (!sub && subs.status !== "Active")}
            <div class="flex flex-wrap gap-2">
              <Button
                icon={Crown}
                onclick={async () => await createCheckoutSession()}
              >
                Subscribe
              </Button>
            </div>
          {/if}

          {#if sub}
            {@const remainingSeats = sub.seats - sub.memberEmails.length}
            {#if remainingSeats <= 0}
              All seats {sub.seats} are in use. Click "Manage" and add more seats
              to add more members.
            {:else}
              You have {remainingSeats} seats remainingSeats. Add more members here.
              <Input
                class="border-muted text-foreground border-2 text-base focus:ring-0 focus-visible:ring-0"
                placeholder="List of members"
                bind:value={memberInput}
                onblur={async () => {
                  await addAndClear();
                }}
                onkeydown={async (e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    await addAndClear();
                  }
                }}
              />
            {/if}

            {#if sub.memberEmails.length > 0}
              <div class="flex flex-wrap pt-2">
                {#each sub.memberEmails as memberEmail (memberEmail)}
                  <div class="space-x-1 text-sm">
                    <span>{memberEmail}</span>
                    {#if memberEmail != auth.user.email}
                      <Button
                        class="m-0 h-4 w-4 p-0 "
                        variant="outlined"
                        onclick={async () => await removeMember(memberEmail)}
                      >
                        <X
                          class="cursor-pointer text-gray-400 hover:text-gray-500"
                        />
                      </Button>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          {/if}
          <div></div>
        </div>
      {/await}
    </SubSection>
  </Section>
</div>
