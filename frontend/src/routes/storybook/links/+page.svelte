<script lang="ts">
  import { Link, type LinkColor, type LinkUnderline } from "$lib/kosui/link";
  import { toTitleCase } from "$lib/kosui/utils";

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
