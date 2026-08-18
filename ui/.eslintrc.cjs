module.exports = {
  root: true,
  parser: "@typescript-eslint/parser",
  plugins: ["@typescript-eslint", "boundaries"],
  settings: {
    "boundaries/elements": [
      { type: "app", pattern: "src/app/*" },
      { type: "pages", pattern: "src/pages/*" },
      { type: "widgets", pattern: "src/widgets/*" },
      { type: "features", pattern: "src/features/*" },
      { type: "entities", pattern: "src/entities/*" },
      { type: "shared", pattern: "src/shared/*" },
    ],
    "boundaries/ignore": ["**/*.test.ts"],
  },
  rules: {
    // Penegakan Aturan Batas Arsitektur Feature-Sliced Design (Inward Only Dependency)
    "boundaries/element-types": [
      "error",
      {
        default: "disallow",
        rules: [
          {
            from: "app",
            allow: ["pages", "widgets", "features", "entities", "shared"],
          },
          {
            from: "pages",
            allow: ["widgets", "features", "entities", "shared"],
          },
          {
            from: "widgets",
            allow: ["features", "entities", "shared"],
          },
          {
            from: "features",
            allow: ["entities", "shared"],
          },
          {
            from: "entities",
            allow: ["shared"],
          },
          {
            from: "shared",
            allow: ["shared"], // Shared tidak boleh import dari entities/features/pages
          },
        ],
      },
    ],
  },
};
