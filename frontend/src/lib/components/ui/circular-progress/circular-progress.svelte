<script lang="ts">
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
  fill="transparent"
  class={classes}
  style="--cp-progress:{progress};"
>
  <circle
    cx="50%"
    cy="50%"
    r="10px"
    stroke-width="2px"
    style="stroke: {trackColor};"
  />
  <circle
    cx="50%"
    cy="50%"
    r="10px"
    stroke-width="2px"
    stroke-linecap="round"
    stroke-dasharray={2 * 3.1415926535 * 10}
    stroke-dashoffset={(1 - progress) * 2 * 3.1415926535 * 10}
    transform-origin="center"
    transform="rotate(-90)"
    stroke={color}
  />
  <text
    x="12"
    y="12"
    font-size="7"
    text-anchor="middle"
    dominant-baseline="central"
    fill={textColor}
  >
    {@render children?.()}
  </text>
</svg>
