export { default as DialogButton } from "./dialog-button.svelte";
export { default as Dialog } from "./dialog.svelte";
export { default as Dialoguer } from "./dialoguer.svelte";

import Dialoguer, { confirm, notice, show } from "./dialoguer.svelte";

export const dialog = {
  confirm,
  notice,
  show,
};

export const dialogBox = {
  dialog: Dialoguer,
};
