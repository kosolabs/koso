import { getContext, setContext } from "svelte";
import { ItemContext } from "../common";

export class CommandContext extends ItemContext {}

export function setCommandContext(state: CommandContext): CommandContext {
  return setContext<CommandContext>(CommandContext, state);
}

export function getCommandContext(): CommandContext {
  const ctx = getContext<CommandContext>(CommandContext);
  if (!ctx) throw new Error("CommandContext is undefined");
  return ctx;
}
