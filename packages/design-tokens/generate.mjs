import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const sourcePath = path.join(repositoryRoot, "schemas", "design-tokens.json");
const generatedTypeScriptPath = path.join(
  repositoryRoot,
  "packages",
  "design-tokens",
  "src",
  "generated.ts",
);
const generatedCssPath = path.join(
  repositoryRoot,
  "packages",
  "design-tokens",
  "src",
  "generated.css",
);

function fail(message) {
  throw new Error(`design-token generation refused: ${message}`);
}

function objectAt(value, label) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    fail(`${label} must be an object`);
  }
  return value;
}

function stringAt(value, label) {
  if (typeof value !== "string" || value.length === 0) {
    fail(`${label} must be a non-empty string`);
  }
  return value;
}

function quote(value) {
  return JSON.stringify(value);
}

function slug(role) {
  return role
    .replaceAll(".", "-")
    .replace(/([a-z0-9])([A-Z])/g, "$1-$2")
    .toLowerCase();
}

function identifier(role) {
  const words = role.split(/[.-]/u);
  return words
    .map((word, index) => {
      if (index === 0) {
        return word;
      }
      return `${word.slice(0, 1).toUpperCase()}${word.slice(1)}`;
    })
    .join("");
}

function pairIdentifier(pair) {
  const background = identifier(pair.background);
  const kind = identifier(pair.kind);
  return `${identifier(pair.foreground)}On${background.slice(0, 1).toUpperCase()}${background.slice(1)}${kind.slice(0, 1).toUpperCase()}${kind.slice(1)}`;
}

function sortedKeys(record) {
  return Object.keys(record).sort((left, right) => left.localeCompare(right));
}

function renderTypeScript(tokens) {
  const themes = objectAt(tokens.themes, "themes");
  const themeNames = sortedKeys(themes);
  if (themeNames.length === 0) {
    fail("themes must not be empty");
  }

  const firstTheme = objectAt(themes[themeNames[0]], `themes.${themeNames[0]}`);
  const firstColors = objectAt(
    firstTheme.colors,
    `themes.${themeNames[0]}.colors`,
  );
  const colorRoles = sortedKeys(firstColors);
  if (colorRoles.length === 0) {
    fail("the first theme must define at least one color role");
  }

  for (const themeName of themeNames) {
    const theme = objectAt(themes[themeName], `themes.${themeName}`);
    const colors = objectAt(theme.colors, `themes.${themeName}.colors`);
    const roles = sortedKeys(colors);
    if (JSON.stringify(roles) !== JSON.stringify(colorRoles)) {
      fail(`theme ${quote(themeName)} does not define the canonical role set`);
    }
    for (const role of roles) {
      stringAt(colors[role], `themes.${themeName}.colors.${role}`);
    }
  }

  const contrastRules = objectAt(tokens.contrastRules, "contrastRules");
  if (!Array.isArray(contrastRules.pairings) || contrastRules.pairings.length === 0) {
    fail("contrastRules.pairings must be a non-empty array");
  }
  const pairings = contrastRules.pairings.map((candidate, index) => {
    const pair = objectAt(candidate, `contrastRules.pairings[${index}]`);
    const foreground = stringAt(
      pair.foreground,
      `contrastRules.pairings[${index}].foreground`,
    );
    const background = stringAt(
      pair.background,
      `contrastRules.pairings[${index}].background`,
    );
    const kind = stringAt(pair.kind, `contrastRules.pairings[${index}].kind`);
    if (!colorRoles.includes(foreground) || !colorRoles.includes(background)) {
      fail(`pairing ${index} references an unknown color role`);
    }
    if (kind !== "text" && kind !== "ui") {
      fail(`pairing ${index} has unsupported kind ${quote(kind)}`);
    }
    return { foreground, background, kind };
  });

  const nonColorChannels = objectAt(tokens.nonColorChannels, "nonColorChannels");
  const channels = objectAt(nonColorChannels.roles, "nonColorChannels.roles");
  const meaningRoles = sortedKeys(channels);
  for (const role of meaningRoles) {
    const channel = objectAt(channels[role], `nonColorChannels.roles.${role}`);
    stringAt(channel.icon, `nonColorChannels.roles.${role}.icon`);
    stringAt(channel.label, `nonColorChannels.roles.${role}.label`);
    stringAt(channel.shape, `nonColorChannels.roles.${role}.shape`);
    if (!colorRoles.includes(role)) {
      fail(`non-color channel ${quote(role)} has no color role`);
    }
  }

  const pairKey = (pair) =>
    `${pair.foreground}|${pair.background}|${pair.kind}`;
  const pairEntries = pairings
    .map(
      (pair) =>
        `  ${quote(pairKey(pair))}: ${quote(`pm-pair-${slug(pair.foreground)}-on-${slug(pair.background)}-${pair.kind}`)},`,
    )
    .join("\n");
  const pairConstants = pairings
    .map(
      (pair) =>
        `  ${pairIdentifier(pair)}: [${quote(pair.foreground)}, ${quote(pair.background)}, ${quote(pair.kind)}] as const,`,
    )
    .join("\n");
  const pairType = pairings
    .map(
      (pair) =>
        `  | readonly [${quote(pair.foreground)}, ${quote(pair.background)}, ${quote(pair.kind)}]`,
    )
    .join("\n");
  const pairKeyType = pairings.map((pair) => quote(pairKey(pair))).join(" | ");
  const roleClasses = meaningRoles
    .map((role) => `  ${quote(role)}: ${quote(`pm-role-${slug(role)}`)},`)
    .join("\n");
  for (const role of meaningRoles) {
    const matchingPairs = pairings.filter(
      (pair) => pair.foreground === role && pair.background === "surface.base",
    );
    if (matchingPairs.length !== 1) {
      fail(
        `meaning role ${quote(role)} needs exactly one declared surface.base pairing`,
      );
    }
  }
  const shapeClasses = meaningRoles
    .map((role) => {
      const channel = channels[role];
      return `  ${quote(role)}: ${quote(`pm-shape-${slug(channel.shape)}`)},`;
    })
    .join("\n");
  const channelEntries = meaningRoles
    .map((role) => {
      const channel = channels[role];
      return `  ${quote(role)}: { icon: ${quote(channel.icon)}, label: ${quote(channel.label)}, shape: ${quote(channel.shape)} },`;
    })
    .join("\n");

  return `// @generated from schemas/design-tokens.json by packages/design-tokens/generate.mjs
// Do not edit by hand. Run \`cargo xtask desktop\` to verify this file.

export const tokenSetVersion = ${quote(stringAt(tokens.tokenSetVersion, "tokenSetVersion"))};
export const sourceSpecVersion = ${quote(stringAt(tokens.specVersion, "specVersion"))};

export type ThemeName = ${themeNames.map(quote).join(" | ")};
export type ColorRole = ${colorRoles.map(quote).join(" | ")};
export type MeaningRole = ${meaningRoles.map(quote).join(" | ")};
export type ContrastKind = "text" | "ui";
export type ContrastPair =
${pairType};
export type TextContrastPair = Extract<
  ContrastPair,
  readonly [string, string, "text"]
>;
type ContrastPairKey = ${pairKeyType};

export interface NonColorChannel {
  readonly icon: string;
  readonly label: string;
  readonly shape: string;
}

export const pairs = {
${pairConstants}
} as const;

const contrastPairClasses: Readonly<Record<ContrastPairKey, string>> = {
${pairEntries}
};

const colorRoleClasses: Readonly<Record<MeaningRole, string>> = {
${roleClasses}
};

const shapeClasses: Readonly<Record<MeaningRole, string>> = {
${shapeClasses}
};

export const nonColorChannels: Readonly<Record<MeaningRole, NonColorChannel>> = {
${channelEntries}
};

function pairVariantClass(prefix: string, pair: ContrastPair): string {
  const key = pair.join("|") as ContrastPairKey;
  return contrastPairClasses[key].replace("pm-pair-", \`pm-\${prefix}-\`);
}

export function pairClass(pair: TextContrastPair): string {
  const key = pair.join("|") as ContrastPairKey;
  return contrastPairClasses[key];
}

export function foregroundClass(pair: TextContrastPair): string {
  return pairVariantClass("foreground", pair);
}

export function backgroundClass(pair: ContrastPair): string {
  return pairVariantClass("background", pair);
}

export function borderClass(pair: ContrastPair): string {
  return pairVariantClass("border", pair);
}

export function outlineClass(pair: ContrastPair): string {
  return pairVariantClass("outline", pair);
}

export function roleClass(role: MeaningRole): string {
  return colorRoleClasses[role];
}

export function shapeClass(role: MeaningRole): string {
  return shapeClasses[role];
}

export function channelFor(role: MeaningRole): NonColorChannel {
  return nonColorChannels[role];
}
`;
}

function renderCss(tokens) {
  const themes = objectAt(tokens.themes, "themes");
  const themeNames = sortedKeys(themes);
  const firstTheme = objectAt(themes[themeNames[0]], `themes.${themeNames[0]}`);
  const colorRoles = sortedKeys(
    objectAt(firstTheme.colors, `themes.${themeNames[0]}.colors`),
  );
  const pairings = objectAt(tokens.contrastRules, "contrastRules").pairings;
  const channels = objectAt(
    objectAt(tokens.nonColorChannels, "nonColorChannels").roles,
    "nonColorChannels.roles",
  );
  const meaningRoles = sortedKeys(channels);
  const shapeDeclarations = {
    "badge-link": [
      "  border-style: solid;",
      "  border-width: 2px;",
      "  border-radius: 0.2rem 0.8rem;",
    ],
    "badge-lock": [
      "  border-style: solid;",
      "  border-width: 3px;",
      "  border-radius: 0.8rem 0.8rem 0.25rem 0.25rem;",
    ],
    "bar-active": ["  border-style: solid;", "  border-width: 3px 2px 2px;"],
    "bar-check": ["  border-style: solid;", "  border-width: 2px 3px 3px;"],
    "bar-warn": ["  border-style: solid;", "  border-width: 3px;"],
    check: ["  border-style: solid;", "  border-width: 2px;"],
    chevron: ["  border-style: solid;", "  border-width: 2px 3px;"],
    "chevron-double": ["  border-style: double;", "  border-width: 4px;"],
    cross: ["  border-style: solid;", "  border-width: 3px;"],
    dot: [
      "  border-style: solid;",
      "  border-width: 2px;",
      "  border-radius: 999px;",
    ],
    "dot-hollow": [
      "  border-style: double;",
      "  border-width: 4px;",
      "  border-radius: 999px;",
    ],
    "rect-dashed": [
      "  border-style: dashed;",
      "  border-width: 2px;",
    ],
    "rect-double": ["  border-style: double;", "  border-width: 4px;"],
    "rect-notched": [
      "  border-style: solid;",
      "  border-width: 2px;",
      "  clip-path: polygon(0 0, calc(100% - 0.75rem) 0, 100% 0.75rem, 100% 100%, 0 100%);",
    ],
    "rect-plain": ["  border-style: solid;", "  border-width: 2px;"],
    "rect-rounded": [
      "  border-style: solid;",
      "  border-width: 2px;",
      "  border-radius: 0.65rem;",
    ],
    "rect-square": [
      "  border-style: solid;",
      "  border-width: 2px;",
      "  border-radius: 0.25rem;",
    ],
    triangle: ["  border-style: solid;", "  border-width: 2px 2px 4px;"],
  };

  const declarationsFor = (themeName) => {
    const colors = objectAt(
      objectAt(themes[themeName], `themes.${themeName}`).colors,
      `themes.${themeName}.colors`,
    );
    const colorScheme = themeName === "light" ? "light" : "dark";
    return [
      `  color-scheme: ${colorScheme};`,
      ...colorRoles.map(
        (role) => `  --pm-color-${slug(role)}: ${stringAt(colors[role], role)};`,
      ),
    ].join("\n");
  };

  const themeBlocks = themeNames
    .map((themeName) => {
      if (themeName === "dark") {
        return `:root,\n[data-theme="dark"] {\n${declarationsFor(themeName)}\n}`;
      }
      return `[data-theme=${quote(themeName)}] {\n${declarationsFor(themeName)}\n}`;
    })
    .join("\n\n");

  const lightSystem = themes.light
    ? `@media (prefers-color-scheme: light) {\n  :root:not([data-theme]) {\n${declarationsFor("light")
        .split("\n")
        .map((line) => `  ${line}`)
        .join("\n")}\n  }\n}`
    : "";

  const roleBlocks = meaningRoles
    .map(
      (role) =>
        `.pm-role-${slug(role)} {\n  --pm-role-color: var(--pm-color-${slug(role)});\n}`,
    )
    .join("\n\n");
  const shapeBlocks = [
    ...new Set(
      meaningRoles.map((role) => {
        const channel = objectAt(channels[role], `nonColorChannels.roles.${role}`);
        return stringAt(channel.shape, `nonColorChannels.roles.${role}.shape`);
      }),
    ),
  ]
    .sort((left, right) => left.localeCompare(right))
    .map((shape) => {
      if (!Object.hasOwn(shapeDeclarations, shape)) {
        fail(`shape ${quote(shape)} has no generated renderer`);
      }
      const declarations = shapeDeclarations[shape];
      return `.pm-shape-${slug(shape)} {\n${declarations.join("\n")}\n}`;
    })
    .join("\n\n");
  const pairBlocks = pairings
    .map((pair) => {
      const foreground = stringAt(pair.foreground, "pairing.foreground");
      const background = stringAt(pair.background, "pairing.background");
      const kind = stringAt(pair.kind, "pairing.kind");
      const suffix = `${slug(foreground)}-on-${slug(background)}-${kind}`;
      const foregroundValue = `var(--pm-color-${slug(foreground)})`;
      const backgroundValue = `var(--pm-color-${slug(background)})`;
      return [
        `.pm-pair-${suffix} {\n  color: ${foregroundValue};\n  background-color: ${backgroundValue};\n}`,
        `.pm-foreground-${suffix} {\n  color: ${foregroundValue};\n}`,
        `.pm-background-${suffix} {\n  background-color: ${backgroundValue};\n}`,
        `.pm-border-${suffix} {\n  border-color: ${foregroundValue};\n}`,
        `.pm-outline-${suffix}:focus-visible {\n  outline: 3px solid ${foregroundValue};\n  outline-offset: 3px;\n}`,
      ].join("\n\n");
    })
    .join("\n\n");

  return `/* @generated from schemas/design-tokens.json by packages/design-tokens/generate.mjs */
/* Do not edit by hand. Run \`cargo xtask desktop\` to verify this file. */

${themeBlocks}

${lightSystem}

${roleBlocks}

${shapeBlocks}

${pairBlocks}
`;
}

async function main() {
  const mode = process.argv[2];
  if (mode !== "--check" && mode !== "--write") {
    fail("usage: node packages/design-tokens/generate.mjs --check|--write");
  }

  const source = await readFile(sourcePath, "utf8");
  const tokens = JSON.parse(source);
  const generatedTypeScript = renderTypeScript(objectAt(tokens, "root"));
  const generatedCss = renderCss(objectAt(tokens, "root"));

  if (mode === "--write") {
    await writeFile(generatedTypeScriptPath, generatedTypeScript, "utf8");
    await writeFile(generatedCssPath, generatedCss, "utf8");
    process.stdout.write("design-token generation: wrote 2 generated files\n");
    return;
  }

  const existingTypeScript = await readFile(generatedTypeScriptPath, "utf8");
  const existingCss = await readFile(generatedCssPath, "utf8");
  const stale = [];
  if (existingTypeScript !== generatedTypeScript) {
    stale.push(path.relative(repositoryRoot, generatedTypeScriptPath));
  }
  if (existingCss !== generatedCss) {
    stale.push(path.relative(repositoryRoot, generatedCssPath));
  }
  if (stale.length > 0) {
    fail(
      `generated output drifted: ${stale.join(", ")}; run npm run tokens:write from apps/desktop`,
    );
  }
  process.stdout.write("design-token generation: 2 generated files are current\n");
}

await main();
