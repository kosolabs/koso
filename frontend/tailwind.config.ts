import type { Config } from "tailwindcss";
import tailwindcssAnimate from "tailwindcss-animate";
import { fontFamily } from "tailwindcss/defaultTheme";

const config: Config = {
  darkMode: ["class"],
  content: ["./src/**/*.{html,js,svelte,ts}"],
  safelist: ["dark"],
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      colors: {
        border: "hsl(var(--border) / <alpha-value>)",
        input: "hsl(var(--input) / <alpha-value>)",
        ring: "hsl(var(--ring) / <alpha-value>)",
        background: "hsl(var(--background) / <alpha-value>)",
        foreground: "hsl(var(--foreground) / <alpha-value>)",
        primary: {
          DEFAULT: "hsl(var(--primary) / <alpha-value>)",
          foreground: "hsl(var(--primary-foreground) / <alpha-value>)",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary) / <alpha-value>)",
          foreground: "hsl(var(--secondary-foreground) / <alpha-value>)",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive) / <alpha-value>)",
          foreground: "hsl(var(--destructive-foreground) / <alpha-value>)",
        },
        muted: {
          DEFAULT: "hsl(var(--muted) / <alpha-value>)",
          foreground: "hsl(var(--muted-foreground) / <alpha-value>)",
        },
        accent: {
          DEFAULT: "hsl(var(--accent) / <alpha-value>)",
          foreground: "hsl(var(--accent-foreground) / <alpha-value>)",
        },
        popover: {
          DEFAULT: "hsl(var(--popover) / <alpha-value>)",
          foreground: "hsl(var(--popover-foreground) / <alpha-value>)",
        },
        card: {
          DEFAULT: "hsl(var(--card) / <alpha-value>)",
          foreground: "hsl(var(--card-foreground) / <alpha-value>)",
        },
        sidebar: {
          DEFAULT: "hsl(var(--sidebar-background))",
          foreground: "hsl(var(--sidebar-foreground))",
          primary: "hsl(var(--sidebar-primary))",
          "primary-foreground": "hsl(var(--sidebar-primary-foreground))",
          accent: "hsl(var(--sidebar-accent))",
          "accent-foreground": "hsl(var(--sidebar-accent-foreground))",
          border: "hsl(var(--sidebar-border))",
          ring: "hsl(var(--sidebar-ring))",
        },
        "md-primary": "rgb(var(--md-sys-color-primary) / <alpha-value>)",
        "md-surface-tint":
          "rgb(var(--md-sys-color-surface-tint) / <alpha-value>)",
        "md-on-primary": "rgb(var(--md-sys-color-on-primary) / <alpha-value>)",
        "md-primary-container":
          "rgb(var(--md-sys-color-primary-container) / <alpha-value>)",
        "md-on-primary-container":
          "rgb(var(--md-sys-color-on-primary-container) / <alpha-value>)",
        "md-secondary": "rgb(var(--md-sys-color-secondary) / <alpha-value>)",
        "md-on-secondary":
          "rgb(var(--md-sys-color-on-secondary) / <alpha-value>)",
        "md-secondary-container":
          "rgb(var(--md-sys-color-secondary-container) / <alpha-value>)",
        "md-on-secondary-container":
          "rgb(var(--md-sys-color-on-secondary-container) / <alpha-value>)",
        "md-tertiary": "rgb(var(--md-sys-color-tertiary) / <alpha-value>)",
        "md-on-tertiary":
          "rgb(var(--md-sys-color-on-tertiary) / <alpha-value>)",
        "md-tertiary-container":
          "rgb(var(--md-sys-color-tertiary-container) / <alpha-value>)",
        "md-on-tertiary-container":
          "rgb(var(--md-sys-color-on-tertiary-container) / <alpha-value>)",
        "md-error": "rgb(var(--md-sys-color-error) / <alpha-value>)",
        "md-on-error": "rgb(var(--md-sys-color-on-error) / <alpha-value>)",
        "md-error-container":
          "rgb(var(--md-sys-color-error-container) / <alpha-value>)",
        "md-on-error-container":
          "rgb(var(--md-sys-color-on-error-container) / <alpha-value>)",
        "md-background": "rgb(var(--md-sys-color-background) / <alpha-value>)",
        "md-on-background":
          "rgb(var(--md-sys-color-on-background) / <alpha-value>)",
        "md-surface": "rgb(var(--md-sys-color-surface) / <alpha-value>)",
        "md-on-surface": "rgb(var(--md-sys-color-on-surface) / <alpha-value>)",
        "md-surface-variant":
          "rgb(var(--md-sys-color-surface-variant) / <alpha-value>)",
        "md-on-surface-variant":
          "rgb(var(--md-sys-color-on-surface-variant) / <alpha-value>)",
        "md-outline": "rgb(var(--md-sys-color-outline) / <alpha-value>)",
        "md-outline-variant":
          "rgb(var(--md-sys-color-outline-variant) / <alpha-value>)",
        "md-shadow": "rgb(var(--md-sys-color-shadow) / <alpha-value>)",
        "md-scrim": "rgb(var(--md-sys-color-scrim) / <alpha-value>)",
        "md-inverse-surface":
          "rgb(var(--md-sys-color-inverse-surface) / <alpha-value>)",
        "md-inverse-on-surface":
          "rgb(var(--md-sys-color-inverse-on-surface) / <alpha-value>)",
        "md-inverse-primary":
          "rgb(var(--md-sys-color-inverse-primary) / <alpha-value>)",
        "md-primary-fixed":
          "rgb(var(--md-sys-color-primary-fixed) / <alpha-value>)",
        "md-on-primary-fixed":
          "rgb(var(--md-sys-color-on-primary-fixed) / <alpha-value>)",
        "md-primary-fixed-dim":
          "rgb(var(--md-sys-color-primary-fixed-dim) / <alpha-value>)",
        "md-on-primary-fixed-variant":
          "rgb(var(--md-sys-color-on-primary-fixed-variant) / <alpha-value>)",
        "md-secondary-fixed":
          "rgb(var(--md-sys-color-secondary-fixed) / <alpha-value>)",
        "md-on-secondary-fixed":
          "rgb(var(--md-sys-color-on-secondary-fixed) / <alpha-value>)",
        "md-secondary-fixed-dim":
          "rgb(var(--md-sys-color-secondary-fixed-dim) / <alpha-value>)",
        "md-on-secondary-fixed-variant":
          "rgb(var(--md-sys-color-on-secondary-fixed-variant) / <alpha-value>)",
        "md-tertiary-fixed":
          "rgb(var(--md-sys-color-tertiary-fixed) / <alpha-value>)",
        "md-on-tertiary-fixed":
          "rgb(var(--md-sys-color-on-tertiary-fixed) / <alpha-value>)",
        "md-tertiary-fixed-dim":
          "rgb(var(--md-sys-color-tertiary-fixed-dim) / <alpha-value>)",
        "md-on-tertiary-fixed-variant":
          "rgb(var(--md-sys-color-on-tertiary-fixed-variant) / <alpha-value>)",
        "md-surface-dim":
          "rgb(var(--md-sys-color-surface-dim) / <alpha-value>)",
        "md-surface-bright":
          "rgb(var(--md-sys-color-surface-bright) / <alpha-value>)",
        "md-surface-container-lowest":
          "rgb(var(--md-sys-color-surface-container-lowest) / <alpha-value>)",
        "md-surface-container-low":
          "rgb(var(--md-sys-color-surface-container-low) / <alpha-value>)",
        "md-surface-container":
          "rgb(var(--md-sys-color-surface-container) / <alpha-value>)",
        "md-surface-container-high":
          "rgb(var(--md-sys-color-surface-container-high) / <alpha-value>)",
        "md-surface-container-highest":
          "rgb(var(--md-sys-color-surface-container-highest) / <alpha-value>)",
      },
      borderRadius: {
        xl: "calc(var(--radius) + 4px)",
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      fontFamily: {
        sans: ["Nunito", ...fontFamily.sans],
      },
      keyframes: {
        "accordion-down": {
          from: { height: "0" },
          to: { height: "var(--bits-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--bits-accordion-content-height)" },
          to: { height: "0" },
        },
        "caret-blink": {
          "0%,70%,100%": { opacity: "1" },
          "20%,50%": { opacity: "0" },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
        "caret-blink": "caret-blink 1.25s ease-out infinite",
      },
    },
  },
  plugins: [tailwindcssAnimate],
};

export default config;
