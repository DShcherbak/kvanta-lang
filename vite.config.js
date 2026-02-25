import { defineConfig } from "vite";

export default defineConfig({
  base: "/kvanta-lang/",
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./tests/setup.js"],
  },
});