<script lang="ts">
  import {
    computePosition,
    type FloatingElement,
    type ReferenceElement,
  } from "@floating-ui/dom";

  let reference: ReferenceElement;
  let tooltip: FloatingElement;

  $: computePosition(reference, tooltip);
</script>

<button bind:this={reference}>A Button</button>
<slot name="trigger"></slot>
<div bind:this={tooltip}>My Tooltip</div>
