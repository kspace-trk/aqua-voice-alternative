// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: [
    '@nuxt/eslint',
    '@nuxt/test-utils/module',
    '@nuxt/image',
  ],
  ssr: false,

  devtools: { enabled: true },

  css: ['~/assets/css/reset.css'],

  vite: {
    css: {
      preprocessorOptions: {
        scss: {
          additionalData: '@use "~/assets/scss/variables.scss" as *;',
        },
      },
    },
  },

  eslint: {
    config: {
      stylistic: {
        indent: 2,
        semi: true,
        quotes: 'single',
      },
    },
  },
});
