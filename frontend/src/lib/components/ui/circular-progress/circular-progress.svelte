<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";

  type Props = {
    children?: Snippet;
    progress: number;
    size?: string | number;
    thickness?: string;
    trackColor?: string;
    textColor?: string;
    color?: string;
    class?: string;
  };

  const {
    children,
    progress,
    size = 24,
    trackColor = "color-mix(in srgb, currentColor 10%, transparent)",
    textColor = "currentColor",
    color = "currentColor",
    class: classes,
  }: Props = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  class={cn("progress-bar", classes)}
  style="--cp-progress:{progress};"
>
  <circle class="track" style="stroke: {trackColor};" />
  <circle class="progress" style="stroke: {color}" />
  <text
    class="text"
    x="12"
    y="12"
    font-size="7"
    text-anchor="middle"
    dominant-baseline="central"
    style="fill: {textColor}"
  >
    {@render children?.()}
  </text>
</svg>

<style>
  .progress-bar {
    fill: transparent;
  }

  .track {
    cx: 50%;
    cy: 50%;
    r: 10px;
    stroke-width: 2px;
  }

  .progress {
    cx: 50%;
    cy: 50%;
    r: 10px;
    stroke-width: 2px;
    stroke-linecap: round;
    stroke-dasharray: calc(2 * 3.1415926535 * 10px);
    stroke-dashoffset: calc((1 - var(--cp-progress)) * 2 * 3.1415926535 * 10px);
    transform-origin: center;
    transform: rotate(-90deg);
  }
</style>
