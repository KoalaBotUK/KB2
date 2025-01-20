module.exports = {
  content: ['./src/**/*.{vue,js,ts}'],
  plugins: [require('daisyui')],
  theme: {
    extend: {
      colors: {
        "discord": {
          light: "color-mix(in srgb, var(--discord-var), white 20%)",
          DEFAULT: "var(--discord-var)",
          dark: "color-mix(in srgb, var(--discord-var), black 20%)"
        },
        "google": {
          light: "color-mix(in srgb, var(--google-var), white 20%)",
          DEFAULT: "var(--google-var)",
          dark: "color-mix(in srgb, var(--google-var), black 20%)"
        },
        "facebook": {
          light: "color-mix(in srgb, var(--facebook-var), white 20%)",
          DEFAULT: "var(--facebook-var)",
          dark: "color-mix(in srgb, var(--facebook-var), black 20%)"
        },
        "github": {
          light: "color-mix(in srgb, var(--github-var), white 20%)",
          DEFAULT: "var(--github-var)",
          dark: "color-mix(in srgb, var(--github-var), black 20%)"
        },
      },
    },
  },
  daisyui: {
    themes: [
      {
        light:
          {
            ...require("daisyui/src/theming/themes")["[data-theme=light]"],
            "--discord-var": "var(--discord)",
            "--google-var": "var(--google)",
            "--facebook-var": "var(--facebook)",
            "--github-var": "var(--github)",
          },
      },
      {
        dark:
          {
            ...require("daisyui/src/theming/themes")["[data-theme=dark]"],
            "--discord-var": "color-mix(in srgb, var(--discord), black 20%)",
            "--google-var": "color-mix(in srgb, var(--google), black 20%)",
            "--facebook-var": "color-mix(in srgb, var(--facebook), black 20%)",
            "--github-var": "color-mix(in srgb, var(--github), black 20%)",
          }
      }
    ]
  },
  safelist: [
    'bg-discord',
  ]
}
;
