import type { Config } from "tailwindcss";
import tailwindcssAnimate from "tailwindcss-animate";
import { fontFamily } from "tailwindcss/defaultTheme";

function m3(prop: string): string {
  return `rgb(from var(--md-sys-color-${prop}) r g b / <alpha-value>)`;
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
        "m3-primary": m3("primary"),
        "m3-surface-tint": m3("surface-tint"),
        "m3-on-primary": m3("on-primary"),
        "m3-primary-container": m3("primary-container"),
        "m3-on-primary-container": m3("on-primary-container"),
        "m3-secondary": m3("secondary"),
        "m3-on-secondary": m3("on-secondary"),
        "m3-secondary-container": m3("secondary-container"),
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
        "m3-surface": m3("surface"),
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
        "m3-surface-container-low": m3("surface-container-low"),
        "m3-surface-container": m3("surface-container"),
        "m3-surface-container-high": m3("surface-container-high"),
        "m3-surface-container-highest": m3("surface-container-highest"),
      },
      borderRadius: {
        m3: "6px",
        xl: "calc(var(--radius) + 4px)",
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      fontFamily: {
        sans: ["Nunito", ...fontFamily.sans],
      },
    },
  },
  plugins: [tailwindcssAnimate],
};

export default config;
