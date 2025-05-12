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

const hoverBaseClasses = ({
  variant,
  color,
}: {
  variant?: Variant;
  color?: Color;
}) =>
  twMerge(
    "disabled:text-m3-on-surface/38 disabled:bg-m3-on-surface/12 disabled:cursor-not-allowed backdrop-blur-sm",

    variant === "filled" && "enabled:hover:opacity-90",
    variant === "tonal" && "enabled:hover:opacity-80",

    variant === "elevated" &&
      color === "primary" &&
      "enabled:hover:bg-m3-primary-container/30",

    variant === "elevated" &&
      color === "secondary" &&
      "enabled:hover:bg-m3-secondary-container/30",

    variant === "elevated" &&
      color === "tertiary" &&
      "enabled:hover:bg-m3-tertiary-container/30",

    variant === "elevated" &&
      color === "error" &&
      "enabled:hover:bg-m3-error-container/30",

    (variant === "outlined" || variant === "plain") &&
      color === "primary" &&
      "enabled:hover:bg-m3-primary/15",

    (variant === "outlined" || variant === "plain") &&
      color === "secondary" &&
      "enabled:hover:bg-m3-secondary/15",

    (variant === "outlined" || variant === "plain") &&
      color === "tertiary" &&
      "enabled:hover:bg-m3-tertiary/15",

    (variant === "outlined" || variant === "plain") &&
      color === "error" &&
      "enabled:hover:bg-m3-error/15",
  );

const focusBaseClasses = ({
  variant,
  color,
}: {
  variant?: Variant;
  color?: Color;
}) =>
  twMerge(
    "disabled:text-m3-on-surface/38 disabled:bg-m3-on-surface/12 disabled:cursor-not-allowed backdrop-blur-sm",

    "focus:ring-1 focus-visible:outline-hidden",
    variant === "filled" && "focus:opacity-90",
    variant === "tonal" && "focus:opacity-80",

    color === "primary" && "focus:ring-m3-primary",
    color === "secondary" && "focus:ring-m3-secondary",
    color === "tertiary" && "focus:ring-m3-tertiary",
    color === "error" && "focus:ring-m3-error",

    variant === "filled" && color === "primary" && "focus:ring-m3-on-primary",

    variant === "filled" &&
      color === "secondary" &&
      "focus:ring-m3-on-secondary",

    variant === "filled" && color === "tertiary" && "focus:ring-m3-on-tertiary",

    variant === "filled" && color === "error" && "focus:ring-m3-on-error",

    variant === "elevated" &&
      color === "primary" &&
      "focus:bg-m3-primary-container/30",

    variant === "elevated" &&
      color === "secondary" &&
      "focus:bg-m3-secondary-container/30",

    variant === "elevated" &&
      color === "tertiary" &&
      "focus:bg-m3-tertiary-container/30",

    variant === "elevated" &&
      color === "error" &&
      "focus:bg-m3-error-container/30",

    (variant === "outlined" || variant === "plain") &&
      color === "primary" &&
      "focus:bg-m3-primary/15",

    (variant === "outlined" || variant === "plain") &&
      color === "secondary" &&
      "focus:bg-m3-secondary/15",

    (variant === "outlined" || variant === "plain") &&
      color === "tertiary" &&
      "focus:bg-m3-tertiary/15",

    (variant === "outlined" || variant === "plain") &&
      color === "error" &&
      "focus:bg-m3-error/15",
  );

export const baseClasses = ({
  variant,
  color,
  shape,
  hover = false,
  focus = false,
}: Variants) =>
  twMerge(
    variant === "elevated" &&
      "bg-m3-surface-container-low shadow shadow-m3-shadow/20",
    variant === "outlined" && "ring-1",
    variant === "plain" && "",

    color === "primary" && "text-m3-primary",
    color === "secondary" && "text-m3-secondary",
    color === "tertiary" && "text-m3-tertiary",
    color === "error" && "text-m3-error",

    shape === "square" && "",
    shape === "rounded" && "rounded-md",
    shape === "circle" && "rounded-full",

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

    hover && hoverBaseClasses({ variant, color }),
    focus && focusBaseClasses({ variant, color }),
  );
