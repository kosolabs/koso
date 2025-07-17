<script lang="ts">
  import { page } from "$app/state";
  import { headers, parseResponse } from "$lib/api";
  import { getAuthContext } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { toast } from "$lib/components/ui/sonner";
  import { deleteUserConnection, redirectToConnectUserFlow } from "$lib/github";
  import { Button } from "$lib/kosui/button";
  import Chip from "$lib/kosui/chip/chip.svelte";
  import { getDialoguerContext } from "$lib/kosui/dialog";
  import { Input } from "$lib/kosui/input";
  import { Link } from "$lib/kosui/link";
  import { CircularProgress } from "$lib/kosui/progress";
  import { ToggleButton, ToggleGroup } from "$lib/kosui/toggle";
  import { toTitleCase } from "$lib/kosui/utils";
  import {
    CircleX,
    Crown,
    Github,
    Moon,
    Send,
    Sun,
    SunMoon,
    Trash2,
  } from "@lucide/svelte";
  import { userPrefersMode as mode } from "mode-watcher";
  import Section from "./section.svelte";
  import SubSection from "./sub-section.svelte";

  const dialog = getDialoguerContext();

  let auth = getAuthContext();
  let profile: Promise<Profile> = $state(load());

  type Notifier = "slack" | "telegram";

  type SlackNotificationConfig = {
    notifier: "slack";
    email: string;
    enabled: boolean;
    settings: {
      userId: string;
    };
  };

  type TelegramNotificationConfig = {
    notifier: "telegram";
    email: string;
    enabled: boolean;
    settings: {
      chatId: number;
    };
  };

  type NotificationConfig =
    | SlackNotificationConfig
    | TelegramNotificationConfig;

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

  async function sendTestNotification(kind: Notifier) {
    const name = toTitleCase(kind);
    const toastId = toast.loading(`Sending test ${name} notification...`);

    try {
      let resp = await fetch(`/api/notifiers/${kind}/test`, {
        method: "POST",
        headers: {
          ...headers(auth),
          "Content-Type": "application/json",
        },
      });
      await parseResponse(auth, resp);
      toast.success(`${name} test notification sent successfully.`, {
        id: toastId,
      });
    } catch {
      toast.error(`Failed to send ${name} test notification.`, { id: toastId });
    }
  }

  async function deleteNotificationConfig(kind: Notifier) {
    const name = toTitleCase(kind);

    if (
      !(await dialog.confirm({
        message,
        title: `Delete ${name} Authorization?`,
        icon: Trash2,
      }))
    ) {
      return;
    }

    const toastId = toast.loading(`Deleting ${name} authorization...`);

    try {
      let resp = await fetch(`/api/notifiers/${kind}`, {
        method: "DELETE",
        headers: {
          ...headers(auth),
          "Content-Type": "application/json",
        },
      });
      await parseResponse(auth, resp);
      toast.success(`${name} authorization deleted.`, { id: toastId });
      profile = load();
    } catch {
      toast.error(`Failed to delete ${name} authorization.`, { id: toastId });
    }
  }

  function getNotificationConfig(
    profile: Profile,
    notifier: "slack",
  ): SlackNotificationConfig | null;
  function getNotificationConfig(
    profile: Profile,
    notifier: "telegram",
  ): TelegramNotificationConfig | null;
  function getNotificationConfig(
    profile: Profile,
    notifier: Notifier,
  ): SlackNotificationConfig | TelegramNotificationConfig | null {
    return (
      profile.notificationConfigs.find(
        (config) => config.notifier === notifier,
      ) || null
    );
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

<Navbar title="Profile">
  {#snippet left()}
    <div>
      <div>
        <h1 class="ml-2 text-lg">Profile Settings for {auth.user.name}</h1>
      </div>
      <div class="text-small ml-2" title="Profile">Account > User Profile</div>
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
      {#if localStorage.getItem("slack-enabled") === "true"}
        <SubSection title="Slack">
          {@const slackConfig = getNotificationConfig(profile, "slack")}
          {#if slackConfig}
            <div class="flex flex-col gap-2">
              <div>Koso is authorized to send messages to Slack.</div>
              <div class="flex flex-wrap gap-2">
                <Button
                  icon={Send}
                  onclick={() => sendTestNotification("slack")}
                >
                  Send Test Slack Notification
                </Button>
                <div class="ml-auto">
                  <Button
                    icon={CircleX}
                    variant="filled"
                    onclick={() => deleteNotificationConfig("slack")}
                  >
                    Delete Slack Authorization
                  </Button>
                </div>
              </div>
              <div></div>
            </div>
          {:else}
            Koso is not authorized to send messages to Slack. To authorize Koso,
            ensure that the Kosobot app is installed in your workspace, then
            send the <code>/token</code> command to
            <Link href="https://slack.com/app_redirect?app=A093QC1FW85">
              @Kosobot
            </Link>.
          {/if}
        </SubSection>
      {/if}

      <SubSection title="Telegram">
        {@const telegramConfig = getNotificationConfig(profile, "telegram")}
        {#if telegramConfig}
          <div class="flex flex-col gap-2">
            <div>Koso is authorized to send messages to Telegram.</div>
            <div class="flex flex-wrap gap-2">
              <Button
                icon={Send}
                onclick={() => sendTestNotification("telegram")}
              >
                Send Test Telegram Notification
              </Button>
              <div class="ml-auto">
                <Button
                  icon={CircleX}
                  variant="filled"
                  onclick={() => deleteNotificationConfig("telegram")}
                >
                  Delete Telegram Authorization
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

  <Section title="Billing and licensing">
    <SubSection title="Overview">
      {#await profile}
        <div class="flex place-content-center items-center gap-2">
          <CircularProgress />
          <div>Loading...</div>
        </div>
      {:then profile}
        {@const subs = profile.subscriptions}

        <div class="flex flex-col gap-2">
          <div>
            {#if subs.status === "None"}
              You're a Koso for Individuals user.
            {:else if subs.status === "Expired"}
              You're a Koso for Individuals user. Your premium subscription
              exired.
            {:else if subs.status === "Active"}
              You're a premium user. Thanks for supporting us!
            {:else}
              Something went wrong. Invalid subscription status "{subs.status}".
              Let us know!
            {/if}
          </div>
          <div></div>
        </div>
      {/await}
    </SubSection>

    <SubSection title="Subscription">
      {#await profile}
        <div class="flex place-content-center items-center gap-2">
          <CircularProgress />
          <div>Loading...</div>
        </div>
      {:then profile}
        {@const sub = profile.subscriptions.ownedSubscription}

        <div class="flex flex-col gap-2">
          <div>
            {#if !sub || sub.status === "None"}
              You do not have an active subscription.
            {:else if sub.status === "Expired"}
              Your subscription is expired.
            {:else if sub.status === "Active"}
              You have a premium subscription.
            {:else}
              Something went wrong. Invalid subscription status "{sub.status}".
              Let us know!
            {/if}
          </div>

          <div>
            {#if sub && sub.status === "Active"}
              <Button
                icon={Crown}
                onclick={async () => await createPortalSession()}
              >
                Manage
              </Button>
            {:else}
              <Button
                icon={Crown}
                onclick={async () => await createCheckoutSession()}
              >
                Subscribe
              </Button>
            {/if}
          </div>

          {#if sub}
            {@const remainingSeats = sub.seats - sub.memberEmails.length}
            <SubSection title="Members">
              <div class="flex flex-col gap-2">
                {#if remainingSeats <= 0}
                  <div>
                    All seats ({sub.seats}) are in use. Need more seats? Click
                    "Manage" to add more seats.
                  </div>
                {:else}
                  <div>
                    You have {remainingSeats}
                    remaining seat{remainingSeats === 1 ? "" : "s"}. Add new
                    setMembers. Press Enter after each entry
                  </div>
                {/if}
                <div>
                  <Input
                    class="border-muted text-foreground border-2 text-base focus:ring-0 focus-visible:ring-0"
                    placeholder="List of members"
                    type="email"
                    disabled={remainingSeats <= 0}
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
                </div>

                {#if sub.memberEmails.length > 0}
                  <div class="flex flex-wrap items-center gap-2 pt-2">
                    {#each sub.memberEmails as memberEmail (memberEmail)}
                      <Chip
                        class="px-3 py-1 text-sm"
                        variant="elevated"
                        shape="circle"
                        onDelete={memberEmail === auth.user.email
                          ? undefined
                          : async () => await removeMember(memberEmail)}
                      >
                        {memberEmail}
                      </Chip>
                    {/each}
                  </div>
                {/if}
              </div>
            </SubSection>
          {/if}

          <div></div>
        </div>
      {/await}
    </SubSection>
  </Section>
</div>
