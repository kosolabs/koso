<script lang="ts">
  import {
    Button,
    type ButtonColor,
    type ButtonVariant,
  } from "$lib/kosui/button";
  import { Link, type LinkColor, type LinkUnderline } from "$lib/kosui/link";
  import { toTitleCase } from "$lib/kosui/utils";
  import { Link2 } from "lucide-svelte";

  const buttonVariants: ButtonVariant[] = [
    "elevated",
    "filled",
    "tonal",
    "outlined",
    "plain",
  ];
  const buttonColors: ButtonColor[] = [
    "primary",
    "secondary",
    "tertiary",
    "error",
  ];

  const linkUnderlines: LinkUnderline[] = ["always", "hover", "none"];
  const linkColors: LinkColor[] = [
    "primary",
    "secondary",
    "tertiary",
    "error",
    "inherit",
  ];
</script>

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each buttonVariants as variant (variant)}
    {#each buttonColors as color (color)}
      {@const title = toTitleCase(variant)}
      <div>
        <div class="mb-2">{title} Buttons ({color})</div>
        <div class="flex flex-wrap gap-2">
          <Button {variant} {color}>{title}</Button>
          <Button {variant} {color} icon={Link2}>{title} with icon</Button>
          <Button disabled {variant} {color}>{title} disabled</Button>
          <Button disabled {variant} {color} icon={Link2}
            >{title} disabled with icon</Button
          >
        </div>
      </div>
    {/each}
  {/each}
</div>

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each linkUnderlines as underline (underline)}
    {@const underlineTitle = toTitleCase(underline)}
    <div>
      <div class="mb-2">Underline {underlineTitle}</div>
      <ul>
        {#each linkColors as color (color)}
          <li>
            Here is a <Link {underline} {color} href="">{color}</Link>
            link.
          </li>
        {/each}
      </ul>
    </div>
  {/each}
</div>
