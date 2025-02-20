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
  const buttonVariantsColors = Object.keys(
    buttonVariants.variants.color,
  ) as Exclude<ButtonVariants["color"], undefined>[];
  const buttonVariantsScales = Object.keys(
    buttonVariants.variants.scale,
  ) as Exclude<ButtonVariants["scale"], undefined>[];

  const linkVariantsUnderlines = Object.keys(
    linkVariants.variants.underline,
  ) as Exclude<LinkVariants["underline"], undefined>[];
  const linkVariantsColors = Object.keys(
    linkVariants.variants.color,
  ) as Exclude<LinkVariants["color"], undefined>[];
  const linkVariantsScales = Object.keys(
    linkVariants.variants.scale,
  ) as Exclude<LinkVariants["scale"], undefined>[];
</script>

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each buttonVariantsVariants as variant}
    {#each buttonVariantsColors as color}
      {#each buttonVariantsScales as scale}
        {@const title = toTitleCase(variant)}
        <div>
          <div class="mb-2">{title} Buttons ({color}, {scale})</div>
          <div class="flex flex-wrap gap-2">
            <Button {variant} {color} {scale}>{title}</Button>
            <Button {variant} {color} {scale} icon={Link2}
              >{title} with icon</Button
            >
            <Button disabled {variant} {color} {scale}>{title} disabled</Button>
            <Button disabled {variant} {color} {scale} icon={Link2}
              >{title} disabled with icon</Button
            >
          </div>
        </div>
      {/each}
    {/each}
  {/each}
</div>

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each linkVariantsUnderlines as underline}
    {#each linkVariantsScales as scale}
      {@const underlineTitle = toTitleCase(underline)}
      <div>
        <div class="mb-2">Underline {underlineTitle} ({scale})</div>
        <ul>
          {#each linkVariantsColors as color}
            <li>
              Here is a <Link {underline} {color} {scale} href="">{color}</Link>
              link.
            </li>
          {/each}
        </ul>
      </div>
    {/each}
  {/each}
</div>
