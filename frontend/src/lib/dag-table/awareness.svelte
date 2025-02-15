<script module lang="ts">
  import { Node } from ".";

  export type User = {
    email: string;
    name: string;
    picture: string;
  };

  export type Awareness = {
    clientId: number;
    sequence: number;
    selected: Node[];
    user: User;
  };

  type Colors = {
    bg: string;
    outline: string;
  };

  const awarenessClasses: Colors[] = [
    { bg: "bg-lime-600", outline: "outline-lime-600" },
    { bg: "bg-emerald-600", outline: "outline-emerald-600" },
    { bg: "bg-cyan-600", outline: "outline-cyan-600" },
    { bg: "bg-sky-600", outline: "outline-sky-600" },
    { bg: "bg-indigo-600", outline: "outline-indigo-600" },
    { bg: "bg-fuchsia-600", outline: "outline-fuchsia-600" },
    { bg: "bg-pink-600", outline: "outline-pink-600" },
    { bg: "bg-red-600", outline: "outline-red-600" },
    { bg: "bg-orange-600", outline: "outline-orange-600" },
    { bg: "bg-yellow-600", outline: "outline-yellow-600" },
    { bg: "bg-green-600", outline: "outline-green-600" },
    { bg: "bg-teal-600", outline: "outline-teal-600" },
    { bg: "bg-blue-600", outline: "outline-blue-600" },
    { bg: "bg-purple-600", outline: "outline-purple-600" },
    { bg: "bg-violet-600", outline: "outline-violet-600" },
    { bg: "bg-rose-600", outline: "outline-rose-600" },
    { bg: "bg-amber-600", outline: "outline-amber-600" },
  ];

  type AwarenessState = {
    clientId: number;
    sequence: number;
    selected: string[];
    user: User;
  };

  export function parseAwarenessStateResponse(response: string): Awareness[] {
    const resp = JSON.parse(response) as AwarenessState[];
    return resp.map((r) => {
      return {
        clientId: r.clientId,
        sequence: r.sequence,
        selected: r.selected.map(Node.parse),
        user: r.user,
      };
    });
  }

  let nextIndex: number = 0;
  const clients: { [email: string]: number } = {};

  function getColor(user: User): Colors {
    if (!(user.email in clients)) {
      clients[user.email] = nextIndex;
      nextIndex = (nextIndex + 1) % awarenessClasses.length;
    }
    return awarenessClasses[clients[user.email]];
  }

  export function getAwarenessBg(users: User[]): string {
    if (users.length === 0) return "";
    if (users.length > 1) return "bg-secondary";
    return getColor(users[0]).bg;
  }

  export function getAwarenessOutline(users: User[]): string {
    if (users.length === 0) return "";
    if (users.length > 1) return "outline-secondary";
    return getColor(users[0]).outline;
  }

  export function getUniqueUsers(awarenesses: Awareness[]): User[] {
    let users = [];
    let emails: Set<string> = new Set();
    for (const awareness of awarenesses) {
      if (!emails.has(awareness.user.email)) {
        users.push(awareness.user);
        emails.add(awareness.user.email);
      }
    }
    return users;
  }
</script>

<script lang="ts">
  import { PlainTooltip } from "$lib/kosui/tooltip";
  import { cn } from "$lib/kosui/utils";

  type Props = {
    users: User[];
  };
  let { users }: Props = $props();

  let label = $derived.by(() => {
    let result = "";
    if (users.length > 0) {
      result += users[0].name;
    }
    if (users.length > 1) {
      result += ` and ${users.length - 1} more`;
    }
    return result;
  });
</script>

{#if users.length > 0}
  <PlainTooltip arrow>
    {#snippet trigger(ref, props)}
      <div
        bind:this={ref.value}
        {...props}
        role="note"
        aria-label={`${label} selected`}
        class={cn(
          "absolute top-0 right-0 rounded-tr rounded-bl px-1 text-xs text-nowrap text-white",
          getAwarenessBg(users),
        )}
      >
        {label}
      </div>
    {/snippet}
    {#snippet children()}
      <div class={cn("flex flex-col gap-1")}>
        {#each users as user}
          <div
            class={cn(
              "rounded p-1 text-xs text-nowrap",
              getAwarenessBg([user]),
            )}
          >
            {user.name} ({user.email})
          </div>
        {/each}
      </div>
    {/snippet}
  </PlainTooltip>
{/if}
