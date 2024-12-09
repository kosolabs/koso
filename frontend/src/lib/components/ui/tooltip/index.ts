import { Tooltip as TooltipPrimitive } from "bits-ui";
import Arrow from "./tooltip-arrow.svelte";
import Content from "./tooltip-content.svelte";

const Root = TooltipPrimitive.Root;
const Trigger = TooltipPrimitive.Trigger;
const Provider = TooltipPrimitive.Provider;
const Portal = TooltipPrimitive.Portal;

export {
  Root,
  Trigger,
  Arrow,
  Content,
  Provider,
  Portal,
  //
  Root as Tooltip,
  Arrow as TooltipArrow,
  Content as TooltipContent,
  Trigger as TooltipTrigger,
  Provider as TooltipProvider,
  Portal as TooltipPortal,
};
