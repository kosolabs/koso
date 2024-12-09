import { Tooltip as TooltipPrimitive } from "bits-ui0";
import Arrow from "./tooltip-arrow.svelte";
import Content from "./tooltip-content.svelte";

const Root = TooltipPrimitive.Root;
const Trigger = TooltipPrimitive.Trigger;

export {
  Root,
  Trigger,
  Content,
  Arrow,
  //
  Root as Tooltip,
  Content as TooltipContent,
  Trigger as TooltipTrigger,
  Arrow as TooltipArrow,
};
