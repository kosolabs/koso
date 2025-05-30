<script lang="ts">
  import Editable from "../editable/editable.svelte";
  import ResponsiveText from "../responsive-text/responsive-text.svelte";

  type Props = {
    value: number | null;
    editable?: boolean;
    onSelect?: (select: number | null) => void;
  };
  let { value = null, editable = true, onSelect }: Props = $props();

  function select(deadline: string | null) {
    let dl = parseDeadline(deadline);
    value = dl;
    onSelect?.(dl);
  }

  function formatDeadline(deadline: number | null): string | null {
    if (deadline === null) {
      return null;
    }
    let d = new Date(deadline);
    return `${d.getUTCFullYear()}-${`${d.getUTCMonth() + 1}`.padStart(2, "0")}-${d.getUTCDate()}`;
  }

  function parseDeadline(deadline: string | null): number | null {
    return deadline === null ? null : Date.parse(deadline);
  }
</script>

{#if editable}
  <Editable
    value={formatDeadline(value) ?? ""}
    onsave={async (value) => select(value.trim() || null)}
    placeholder="Unset"
    type="date"
    renderValue={(value) => value.replaceAll("-", "/")}
    class="text-m3-on-surface"
  />
{:else}
  <div class="flex items-center gap-2" title={`${value ?? "Unset"}`}>
    <ResponsiveText
      >{value === null
        ? "Unset"
        : formatDeadline(value)?.replaceAll("-", "/")}</ResponsiveText
    >
  </div>
{/if}
