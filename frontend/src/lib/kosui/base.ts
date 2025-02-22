import { tv } from "tailwind-variants";

export const baseVariants = tv({
  base: "disabled:text-m3-on-surface/38 shadow-m3-shadow/20 disabled:bg-m3-on-surface/12 rounded-m3 focus-visible:ring-1 focus-visible:outline-hidden disabled:cursor-not-allowed",
  variants: {
    variant: {
      elevated: "not-disabled:not-active:shadow",
      filled: "",
      tonal: "",
      outlined: "ring-1 focus-visible:ring-2",
      plain: "",
    },
    color: {
      primary: "text-m3-primary focus-visible:ring-m3-primary",
      secondary: "text-m3-secondary focus-visible:ring-m3-secondary",
      tertiary: "text-m3-tertiary focus-visible:ring-m3-tertiary",
      error: "text-m3-error focus-visible:ring-m3-error",
    },
    scale: {
      sm: "text-sm",
      md: "",
      lg: "text-lg",
    },
  },
  compoundVariants: [
    {
      variant: "filled",
      color: "primary",
      class: "bg-m3-primary text-m3-on-primary",
    },
    {
      variant: "filled",
      color: "secondary",
      class: "bg-m3-secondary text-m3-on-secondary",
    },
    {
      variant: "filled",
      color: "tertiary",
      class: "bg-m3-tertiary text-m3-on-tertiary",
    },
    {
      variant: "filled",
      color: "error",
      class: "bg-m3-error text-m3-on-error",
    },
    {
      variant: "tonal",
      color: "primary",
      class: "bg-m3-primary-container text-m3-on-primary-container",
    },
    {
      variant: "tonal",
      color: "secondary",
      class: "bg-m3-secondary-container text-m3-on-secondary-container",
    },
    {
      variant: "tonal",
      color: "tertiary",
      class: "bg-m3-tertiary-container text-m3-on-tertiary-container",
    },
    {
      variant: "tonal",
      color: "error",
      class: "bg-m3-error-container text-m3-on-error-container",
    },
  ],
});
