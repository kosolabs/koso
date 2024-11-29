import { Tooltip as TooltipPrimitive } from "bits-ui";
import Arrow from "./tooltip-arrow.svelte";
import Content from "./tooltip-content.svelte";

const Root = TooltipPrimitive.Root;
const Trigger = TooltipPrimitive.Trigger;
const Provider = TooltipPrimitive.Provider;
const Portal = TooltipPrimitive.Portal;

export {
  Arrow,
  Content,
  Portal,
  Provider,
  Root,
  //
  Root as Tooltip,
  Arrow as TooltipArrow,
  Content as TooltipContent,
  Portal as TooltipPortal,
  Provider as TooltipProvider,
  Trigger as TooltipTrigger,
  Trigger,
};
