<script lang="ts">
  import { colors, underlines } from "$lib/kosui/base";
  import { Link } from "$lib/kosui/link";
  import { toTitleCase } from "$lib/kosui/utils";
</script>

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each underlines as underline (underline)}
    {@const underlineTitle = toTitleCase(underline)}
    <div>
      <div class="mb-2">Underline {underlineTitle}</div>
      <ul>
        {#each colors as color (color)}
          <li>
            Here is a <Link {underline} {color} href="">{color}</Link>
            link.
          </li>
        {/each}
      </ul>
    </div>
  {/each}
</div>
