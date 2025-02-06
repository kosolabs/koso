export { default as Dialog } from "./dialog.svelte";
export { default as Dialoguer } from "./dialoguer.svelte";

import { confirm, notice, show } from "./dialoguer.svelte";

export const dialog = {
  confirm,
  notice,
  show,
};
