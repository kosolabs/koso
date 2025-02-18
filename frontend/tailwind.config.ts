import type { Config } from "tailwindcss";
import tailwindcssAnimate from "tailwindcss-animate";
import { fontFamily } from "tailwindcss/defaultTheme";

function m3(prop: string): string {
  return `rgb(from var(--md-sys-color-${prop}) r g b / <alpha-value>)`;
}

function mix(m3Base: string, m3Layer: string, amount: number): string {
  return `color-mix(in srgb, var(--md-sys-color-${m3Base}), var(--md-sys-color-${m3Layer}) ${amount}%);`;
}

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
        "m3-primary": {
          DEFAULT: m3("primary"),
          hover: mix("primary", "on-primary", 8),
          focus: mix("primary", "on-primary", 10),
          active: mix("primary", "on-primary", 12),
        },
        "m3-surface-tint": m3("surface-tint"),
        "m3-on-primary": m3("on-primary"),
        "m3-primary-container": m3("primary-container"),
        "m3-on-primary-container": m3("on-primary-container"),
        "m3-secondary": m3("secondary"),
        "m3-on-secondary": m3("on-secondary"),
        "m3-secondary-container": {
          DEFAULT: m3("secondary-container"),
          hover: mix("secondary-container", "on-secondary-container", 8),
          focus: mix("secondary-container", "on-secondary-container", 10),
          active: mix("secondary-container", "on-secondary-container", 12),
        },
        "m3-on-secondary-container": m3("on-secondary-container"),
        "m3-tertiary": m3("tertiary"),
        "m3-on-tertiary": m3("on-tertiary"),
        "m3-tertiary-container": m3("tertiary-container"),
        "m3-on-tertiary-container": m3("on-tertiary-container"),
        "m3-error": m3("error"),
        "m3-on-error": m3("on-error"),
        "m3-error-container": m3("error-container"),
        "m3-on-error-container": m3("on-error-container"),
        "m3-background": m3("background"),
        "m3-on-background": m3("on-background"),
        "m3-surface": {
          DEFAULT: m3("surface"),
          hover: mix("surface", "on-surface", 8),
          focus: mix("surface", "on-surface", 10),
          active: mix("surface", "on-surface", 12),
        },
        "m3-on-surface": m3("on-surface"),
        "m3-surface-variant": m3("surface-variant"),
        "m3-on-surface-variant": m3("on-surface-variant"),
        "m3-outline": m3("outline"),
        "m3-outline-variant": m3("outline-variant"),
        "m3-shadow": m3("shadow"),
        "m3-scrim": m3("scrim"),
        "m3-inverse-surface": m3("inverse-surface"),
        "m3-inverse-on-surface": m3("inverse-on-surface"),
        "m3-inverse-primary": m3("inverse-primary"),
        "m3-primary-fixed": m3("primary-fixed"),
        "m3-on-primary-fixed": m3("on-primary-fixed"),
        "m3-primary-fixed-dim": m3("primary-fixed-dim"),
        "m3-on-primary-fixed-variant": m3("on-primary-fixed-variant"),
        "m3-secondary-fixed": m3("secondary-fixed"),
        "m3-on-secondary-fixed": m3("on-secondary-fixed"),
        "m3-secondary-fixed-dim": m3("secondary-fixed-dim"),
        "m3-on-secondary-fixed-variant": m3("on-secondary-fixed-variant"),
        "m3-tertiary-fixed": m3("tertiary-fixed"),
        "m3-on-tertiary-fixed": m3("on-tertiary-fixed"),
        "m3-tertiary-fixed-dim": m3("tertiary-fixed-dim"),
        "m3-on-tertiary-fixed-variant": m3("on-tertiary-fixed-variant"),
        "m3-surface-dim": m3("surface-dim"),
        "m3-surface-bright": m3("surface-bright"),
        "m3-surface-container-lowest": m3("surface-container-lowest"),
        "m3-surface-container-low": {
          DEFAULT: m3("surface-container-low"),
          hover: mix("surface-container-low", "primary", 8),
          focus: mix("surface-container-low", "primary", 10),
          active: mix("surface-container-low", "primary", 12),
        },
        "m3-surface-container": m3("surface-container"),
        "m3-surface-container-high": m3("surface-container-high"),
        "m3-surface-container-highest": m3("surface-container-highest"),
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
