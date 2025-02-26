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
};

export function baseClasses({ variant, color, shape }: Variants) {
  return twMerge(
    "shadow-m3-shadow/20",
    variant === "elevated" && "bg-m3-surface-container-low shadow",
    variant === "filled" && "",
    variant === "tonal" && "",
    variant === "outlined" && "ring-1",
    variant === "plain" && "",

    shape === "square" && "",
    shape === "rounded" && "rounded-m3",
    shape === "circle" && "rounded-full",

    color === "primary" && "text-m3-primary",
    color === "secondary" && "text-m3-secondary",
    color === "tertiary" && "text-m3-tertiary",
    color === "error" && "text-m3-error",

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
  );
}
