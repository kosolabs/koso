<script module lang="ts">
  import type { DetailPanelState } from "$lib/components/ui/detail-panel";
  import { useLocalStorage, type Storable } from "$lib/stores.svelte";
  import { getContext, setContext } from "svelte";

  export class PrefsContext {
    #detailPanel: Storable<DetailPanelState>;

    constructor() {
      this.#detailPanel = useLocalStorage<DetailPanelState>(
        "detail-panel",
        "none",
      );
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
  import type { Snippet } from "svelte";

  type Props = {
    children: Snippet;
  };
  let { children }: Props = $props();

  const ctx = new PrefsContext();
  setPrefsContext(ctx);
</script>

{@render children()}
