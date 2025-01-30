<script lang="ts">
  import { cn } from "$lib/utils";

  type Props = {
    size?: string | number;
    thickness?: string;
    trackColor?: string;
    color?: string;
    class?: string;
  };

  const {
    size = 24,
    trackColor = "color-mix(in srgb, currentColor 10%, transparent)",
    color = "currentColor",
    class: classes,
  }: Props = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  class={cn("circular", classes)}
>
  <circle class="track" style="stroke: {trackColor};" />
  <circle class="path" style="stroke: {color}"></circle>
</svg>

<style>
  .circular {
    fill: transparent;
    animation: rotate 2s linear infinite;
  }

  .track {
    cx: 50%;
    cy: 50%;
    r: 10px;
    stroke-width: 2px;
  }

  .path {
    cx: 50%;
    cy: 50%;
    r: 10px;
    stroke-width: 2px;
    stroke-miterlimit: 4px;
    stroke-dasharray: 1, 200;
    stroke-dashoffset: 0;
    animation: dash 1.5s ease-in-out infinite;
    stroke-linecap: round;
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
