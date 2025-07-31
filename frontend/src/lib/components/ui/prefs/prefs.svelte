<script module lang="ts">
  import { getRegistryContext } from "$lib/components/ui/command-palette";
  import type { DetailPanelState } from "$lib/components/ui/detail-panel";
  import { useLocalStorage, type Storable } from "$lib/stores.svelte";
  import { Bug, BugOff } from "@lucide/svelte";
  import { Action } from "kosui";
  import type { Snippet } from "svelte";
  import { getContext, onMount, setContext } from "svelte";

  export class PrefsContext {
    #debug: Storable<boolean>;
    #detailPanel: Storable<DetailPanelState>;

    constructor() {
      this.#debug = useLocalStorage("debug", false);
      this.#detailPanel = useLocalStorage("detail-panel", "none");
    }

    get debug() {
      return this.#debug.value;
    }

    set debug(value: boolean) {
      this.#debug.value = value;
    }

    get detailPanel() {
      return this.#detailPanel.value;
    }

    set detailPanel(value: DetailPanelState) {
      this.#detailPanel.value = value;
    }
  }

  export function setPrefsContext(ctx: PrefsContext): PrefsContext {
    return setContext<PrefsContext>(PrefsContext, ctx);
  }

  export function getPrefsContext(): PrefsContext {
    const ctx = getContext<PrefsContext>(PrefsContext);
    if (!ctx) throw new Error("PrefsContext is undefined");
    return ctx;
  }
</script>

<script lang="ts">
  type Props = {
    children: Snippet;
  };
  let { children }: Props = $props();

  const ctx = new PrefsContext();
  setPrefsContext(ctx);

  const command = getRegistryContext();

  const actions: Action[] = [
    new Action({
      id: "EnableDebug",
      callback: () => (ctx.debug = true),
      category: "Dev",
      name: "Enable Debug",
      description: "Turn on Debug mode (for developers)",
      icon: Bug,
      enabled: () => localStorage.getItem("dev") === "true",
      selected: () => ctx.debug,
    }),
    new Action({
      id: "DisableDebug",
      callback: () => (ctx.debug = false),
      category: "Dev",
      name: "Disable Debug",
      description: "Turn off Debug mode (for developers)",
      icon: BugOff,
      enabled: () => localStorage.getItem("dev") === "true",
      selected: () => !ctx.debug,
    }),
  ];

  onMount(() => {
    return command.register(...actions);
  });
</script>

{@render children()}
