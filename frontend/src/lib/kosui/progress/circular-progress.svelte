<script lang="ts">
  import type { Snippet } from "svelte";

  const DASH_ARRAY = 2 * 3.1415926535 * 10;

  type Props = {
    children?: Snippet;
    progress?: number;
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
    class: className,
  }: Props = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="transparent"
  class={className}
  class:circular={progress === undefined}
>
  <circle cx="50%" cy="50%" r="10px" stroke={trackColor} stroke-width="2px" />
  <circle
    class:path={progress === undefined}
    cx="50%"
    cy="50%"
    r="10px"
    stroke={color}
    stroke-width="2px"
    stroke-linecap="round"
    stroke-dasharray={DASH_ARRAY}
    stroke-dashoffset={(1 - (progress ?? 1)) * DASH_ARRAY}
    transform-origin="center"
    transform="rotate(-90)"
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

<style>
  .circular {
    animation: rotate 2s linear infinite;
  }

  .path {
    animation: dash 1.5s ease-in-out infinite;
  }

  @keyframes rotate {
    100% {
      transform: rotate(360deg);
    }
  }
  @keyframes dash {
    0% {
      stroke-dasharray: 1, 200;
      stroke-dashoffset: 0;
    }
    100% {
      stroke-dasharray: 89, 200;
      stroke-dashoffset: -62;
    }
  }
</style>
