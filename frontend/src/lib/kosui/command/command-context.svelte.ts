import { getContext, setContext } from "svelte";
import { ItemContext } from "../common";

export class CommandContext extends ItemContext {}

export function setCommandContext(state: CommandContext): CommandContext {
  return setContext<CommandContext>(CommandContext, state);
}

export function getCommandContext(): CommandContext {
  return getContext<CommandContext>(CommandContext);
}
