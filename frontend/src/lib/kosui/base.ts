import { twMerge } from "tailwind-merge";

export const variants = [
  "elevated",
  "filled",
  "tonal",
  "outlined",
  "plain",
] as const;

export const colors = ["primary", "secondary", "tertiary", "error"] as const;
export const shapes = ["square", "rounded", "circle"] as const;

export type Variant = (typeof variants)[number];
export type Color = (typeof colors)[number];
export type Shape = (typeof shapes)[number];

export type Variants = {
  variant?: Variant;
  color?: Color;
  shape?: Shape;
  hover?: boolean;
  focus?: boolean;
};

export const baseClasses = ({
  variant,
  color,
  shape,
  hover = false,
  focus = false,
}: Variants) =>
  twMerge(
    // Base
    "shadow-m3-shadow/20",

    // Interactive base
    "disabled:text-m3-on-surface/38 disabled:bg-m3-on-surface/12 disabled:cursor-not-allowed backdrop-blur-sm",

    shape === "square" && "",
    shape === "rounded" && "rounded-md",
    shape === "circle" && "rounded-full",

    focus && "focus:ring-1 focus-visible:outline-hidden",

    variant === "elevated" && "bg-m3-surface-container-low shadow",
    variant === "outlined" && "ring-1",
    variant === "plain" && "",

    hover && variant === "filled" && "enabled:hover:opacity-90",
    hover && variant === "tonal" && "enabled:hover:opacity-80",

    focus && variant === "filled" && "focus:opacity-90",
    focus && variant === "tonal" && "focus:opacity-80",

    color === "primary" && "text-m3-primary",
    color === "secondary" && "text-m3-secondary",
    color === "tertiary" && "text-m3-tertiary",
    color === "error" && "text-m3-error",

    focus && color === "primary" && "focus:ring-m3-primary",
    focus && color === "secondary" && "focus:ring-m3-secondary",
    focus && color === "tertiary" && "focus:ring-m3-tertiary",
    focus && color === "error" && "focus:ring-m3-error",

    variant === "filled" &&
      color === "primary" &&
      "bg-m3-primary text-m3-on-primary",
    variant === "filled" &&
      color === "secondary" &&
      "bg-m3-secondary text-m3-on-secondary",
    variant === "filled" &&
      color === "tertiary" &&
      "bg-m3-tertiary text-m3-on-tertiary",
    variant === "filled" && color === "error" && "bg-m3-error text-m3-on-error",

    focus &&
      variant === "filled" &&
      color === "primary" &&
      "focus:ring-m3-on-primary",
    focus &&
      variant === "filled" &&
      color === "secondary" &&
      "focus:ring-m3-on-secondary",
    focus &&
      variant === "filled" &&
      color === "tertiary" &&
      "focus:ring-m3-on-tertiary",
    focus &&
      variant === "filled" &&
      color === "error" &&
      "focus:ring-m3-on-error",

    variant === "tonal" &&
      color === "primary" &&
      "bg-m3-primary-container text-m3-on-primary-container",
    variant === "tonal" &&
      color === "secondary" &&
      "bg-m3-secondary-container text-m3-on-secondary-container",
    variant === "tonal" &&
      color === "tertiary" &&
      "bg-m3-tertiary-container text-m3-on-tertiary-container",
    variant === "tonal" &&
      color === "error" &&
      "bg-m3-error-container text-m3-on-error-container",

    hover &&
      variant === "elevated" &&
      color === "primary" &&
      "enabled:hover:bg-m3-primary-container/30",
    hover &&
      variant === "elevated" &&
      color === "secondary" &&
      "enabled:hover:bg-m3-secondary-container/30",
    hover &&
      variant === "elevated" &&
      color === "tertiary" &&
      "enabled:hover:bg-m3-tertiary-container/30",
    hover &&
      variant === "elevated" &&
      color === "error" &&
      "enabled:hover:bg-m3-error-container/30",

    focus &&
      variant === "elevated" &&
      color === "primary" &&
      "focus:bg-m3-primary-container/30",
    focus &&
      variant === "elevated" &&
      color === "secondary" &&
      "focus:bg-m3-secondary-container/30",
    focus &&
      variant === "elevated" &&
      color === "tertiary" &&
      "focus:bg-m3-tertiary-container/30",
    focus &&
      variant === "elevated" &&
      color === "error" &&
      "focus:bg-m3-error-container/30",

    hover &&
      (variant === "outlined" || variant === "plain") &&
      color === "primary" &&
      "enabled:hover:bg-m3-primary/15",
    hover &&
      (variant === "outlined" || variant === "plain") &&
      color === "secondary" &&
      "enabled:hover:bg-m3-secondary/15",
    hover &&
      (variant === "outlined" || variant === "plain") &&
      color === "tertiary" &&
      "enabled:hover:bg-m3-tertiary/15",
    hover &&
      (variant === "outlined" || variant === "plain") &&
      color === "error" &&
      "enabled:hover:bg-m3-error/15",

    focus &&
      (variant === "outlined" || variant === "plain") &&
      color === "primary" &&
      "focus:bg-m3-primary/15",
    focus &&
      (variant === "outlined" || variant === "plain") &&
      color === "secondary" &&
      "focus:bg-m3-secondary/15",
    focus &&
      (variant === "outlined" || variant === "plain") &&
      color === "tertiary" &&
      "focus:bg-m3-tertiary/15",
    focus &&
      (variant === "outlined" || variant === "plain") &&
      color === "error" &&
      "focus:bg-m3-error/15",
  );
