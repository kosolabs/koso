<script lang="ts">
  import {
    Button,
    buttonVariants,
    type ButtonVariants,
  } from "$lib/kosui/button";
  import { Link, linkVariants, type LinkVariants } from "$lib/kosui/link";
  import { toTitleCase } from "$lib/kosui/utils";
  import { Link2 } from "lucide-svelte";

  const buttonVariantsVariants = Object.keys(
    buttonVariants.variants.variant,
  ) as Exclude<ButtonVariants["variant"], undefined>[];

  const linkVariantsUnderlines = Object.keys(
    linkVariants.variants.underline,
  ) as Exclude<LinkVariants["underline"], undefined>[];
  const linkVariantsColors = Object.keys(
    linkVariants.variants.color,
  ) as Exclude<LinkVariants["color"], undefined>[];
</script>

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each buttonVariantsVariants as variant}
    {@const title = toTitleCase(variant)}
    <div>
      <div class="mb-2">{title} Buttons</div>
      <div class="flex flex-wrap gap-2">
        <Button {variant}>{title}</Button>
        <Button {variant} icon={Link2}>{title} with icon</Button>
        <Button disabled {variant}>{title} disabled</Button>
        <Button disabled {variant} icon={Link2}
          >{title} disabled with icon</Button
        >
      </div>
    </div>
  {/each}
</div>

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each linkVariantsUnderlines as underline}
    {@const underlineTitle = toTitleCase(underline)}
    <div>
      <div class="mb-2">Underline {underlineTitle}</div>
      <ul>
        {#each linkVariantsColors as color}
          <li>
            Here is a <Link {underline} {color} href="">{color}</Link> link.
          </li>
        {/each}
      </ul>
    </div>
  {/each}
</div>
