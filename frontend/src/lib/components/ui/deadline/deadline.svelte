<script lang="ts">
  import Editable from "../editable/editable.svelte";
  import ResponsiveText from "../responsive-text/responsive-text.svelte";

  type Props = {
    value: number | null;
    editable: boolean;
    onSelect: (select: number | null) => void;
  };
  let { value, editable, onSelect }: Props = $props();

  function select(deadline: string | null) {
    onSelect(parseDeadline(deadline));
  }

  const UNSET_VALUE = "Unset";

  function formatDeadlineDisplayValue(deadline: number | null): string {
    if (deadline === null) {
      return UNSET_VALUE;
    }
    let d = new Date(deadline);
    return `${pad(d.getUTCMonth() + 1, 2)}/${pad(d.getUTCDate(), 2)}/${pad(d.getUTCFullYear(), 4)}`;
  }

  /** Formats a deadline, milliseconds since epoch, to the form YYYY-MM-DD */
  function formatDeadlineInputValue(deadline: number | null): string {
    if (deadline === null) {
      return "";
    }
    let d = new Date(deadline);
    return `${pad(d.getUTCFullYear(), 4)}-${pad(d.getUTCMonth() + 1, 2)}-${pad(d.getUTCDate(), 2)}`;
  }

  function pad(s: number, len: number) {
    return `${s}`.padStart(len, "0");
  }

  /** Parse deadlines of the form YYYY-MM-DD */
  function parseDeadline(deadline: string | null): number | null {
    return deadline === null ? null : Date.parse(deadline);
  }
</script>

{#if editable}
  <Editable
    value={formatDeadlineInputValue(value)}
    onsave={async (value) => select(value.trim() || null)}
    placeholder={UNSET_VALUE}
    type="date"
    renderValue={(value) => formatDeadlineDisplayValue(parseDeadline(value))}
    class="text-m3-on-surface"
  />
{:else}
  {@const deadline = formatDeadlineDisplayValue(value)}
  <div class="flex items-center gap-2" title={`${deadline}`}>
    <ResponsiveText>{deadline}</ResponsiveText>
  </div>
{/if}
