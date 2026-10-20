import { defineConfig } from 'vite';

// `base: './'` keeps every asset path relative, so the built page works from
// a GitHub Pages project path (/buffoonery/) as well as from the root.
export default defineConfig({
  base: './',
  build: { target: 'es2022' },
});
