<script module lang="ts">
  import { tv, type VariantProps } from "tailwind-variants";

  const segmentedButtonsVariants = tv({
    base: "bg-md-background outline-md-outline text-md-on-surface h-10 px-3 outline first:rounded-l-full last:rounded-r-full",
    variants: {
      variant: {
        enabled: "",
        selected: "bg-md-secondary-container text-md-on-secondary-container",
      },
    },
  });

  type SegmentedButtonsVariantProps = VariantProps<
    typeof segmentedButtonsVariants
  >;

  export type SegmentedButton = {
    name: string;
    value: string;
    enabled?: boolean;
  };

  export type SegmentedButtonsProps = {
    value?: string;
    onSelect?: (value: string) => void;
    buttons: SegmentedButton[];
  };
</script>

<script lang="ts">
  let {
    value = $bindable(),
    onSelect,
    buttons,
  }: SegmentedButtonsProps = $props();

  function handleSelect(newValue: string) {
    value = newValue;
    onSelect?.(value);
  }
</script>

<div class="column m-0 grid w-max auto-cols-fr grid-flow-col p-0">
  {#each buttons as button}
    {@const variant = value === button.value ? "selected" : "enabled"}
    <button
      class={segmentedButtonsVariants({ variant, class: "" })}
      onclick={() => handleSelect(button.value)}
    >
      {button.name}
    </button>
  {/each}
</div>
