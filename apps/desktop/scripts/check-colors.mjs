import { readdir, readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const repositoryRoot = fileURLToPath(new URL("../../../", import.meta.url));
const sourceRoots = [
  path.join(repositoryRoot, "apps", "desktop", "src"),
  path.join(repositoryRoot, "packages", "ui", "src"),
];
const sourceExtensions = new Set([".css", ".html", ".js", ".mjs", ".svg", ".ts", ".tsx"]);

const cssNamedColors = [
  "aliceblue",
  "antiquewhite",
  "aqua",
  "aquamarine",
  "azure",
  "beige",
  "bisque",
  "black",
  "blanchedalmond",
  "blue",
  "blueviolet",
  "brown",
  "burlywood",
  "cadetblue",
  "chartreuse",
  "chocolate",
  "coral",
  "cornflowerblue",
  "cornsilk",
  "crimson",
  "cyan",
  "darkblue",
  "darkcyan",
  "darkgoldenrod",
  "darkgray",
  "darkgreen",
  "darkgrey",
  "darkkhaki",
  "darkmagenta",
  "darkolivegreen",
  "darkorange",
  "darkorchid",
  "darkred",
  "darksalmon",
  "darkseagreen",
  "darkslateblue",
  "darkslategray",
  "darkslategrey",
  "darkturquoise",
  "darkviolet",
  "deeppink",
  "deepskyblue",
  "dimgray",
  "dimgrey",
  "dodgerblue",
  "firebrick",
  "floralwhite",
  "forestgreen",
  "fuchsia",
  "gainsboro",
  "ghostwhite",
  "gold",
  "goldenrod",
  "gray",
  "green",
  "greenyellow",
  "grey",
  "honeydew",
  "hotpink",
  "indianred",
  "indigo",
  "ivory",
  "khaki",
  "lavender",
  "lavenderblush",
  "lawngreen",
  "lemonchiffon",
  "lightblue",
  "lightcoral",
  "lightcyan",
  "lightgoldenrodyellow",
  "lightgray",
  "lightgreen",
  "lightgrey",
  "lightpink",
  "lightsalmon",
  "lightseagreen",
  "lightskyblue",
  "lightslategray",
  "lightslategrey",
  "lightsteelblue",
  "lightyellow",
  "lime",
  "limegreen",
  "linen",
  "magenta",
  "maroon",
  "mediumaquamarine",
  "mediumblue",
  "mediumorchid",
  "mediumpurple",
  "mediumseagreen",
  "mediumslateblue",
  "mediumspringgreen",
  "mediumturquoise",
  "mediumvioletred",
  "midnightblue",
  "mintcream",
  "mistyrose",
  "moccasin",
  "navajowhite",
  "navy",
  "oldlace",
  "olive",
  "olivedrab",
  "orange",
  "orangered",
  "orchid",
  "palegoldenrod",
  "palegreen",
  "paleturquoise",
  "palevioletred",
  "papayawhip",
  "peachpuff",
  "peru",
  "pink",
  "plum",
  "powderblue",
  "purple",
  "rebeccapurple",
  "red",
  "rosybrown",
  "royalblue",
  "saddlebrown",
  "salmon",
  "sandybrown",
  "seagreen",
  "seashell",
  "sienna",
  "silver",
  "skyblue",
  "slateblue",
  "slategray",
  "slategrey",
  "snow",
  "springgreen",
  "steelblue",
  "tan",
  "teal",
  "thistle",
  "tomato",
  "transparent",
  "turquoise",
  "violet",
  "wheat",
  "white",
  "whitesmoke",
  "yellow",
  "yellowgreen",
];

const namedColorPattern = new RegExp(
  `(?<![\\w-])(?:${cssNamedColors.join("|")})(?![\\w-])`,
  "iu",
);
const generatedClassPattern =
  /(?<![-\w])pm-(?:background|border|foreground|outline|pair|role|shape)-(?!color(?:\b|-))[\w-]+/u;
const rawColorVariablePattern = /--pm-color-[\w-]+/u;
const literalPatterns = [
  { label: "hex color", pattern: /#[0-9a-f]{3,8}\b/iu },
  {
    label: "functional color",
    pattern: /\b(?:color|hsl|hsla|hwb|lab|lch|oklab|oklch|rgb|rgba)\s*\(/iu,
  },
  { label: "named CSS color", pattern: namedColorPattern },
];

const scannerRegressionCases = [
  {
    description: "a named color value is rejected",
    pattern: namedColorPattern,
    source: "color: white;",
    expected: true,
  },
  {
    description: "a CSS property containing a color name is allowed",
    pattern: namedColorPattern,
    source: "white-space: nowrap;",
    expected: false,
  },
  {
    description: "a generated class literal is rejected",
    pattern: generatedClassPattern,
    source: '"pm-role-entity-device"',
    expected: true,
  },
  {
    description: "the generated role custom property is allowed",
    pattern: generatedClassPattern,
    source: "color: var(--pm-role-color);",
    expected: false,
  },
  {
    description: "a generated raw color variable use is rejected",
    pattern: rawColorVariablePattern,
    source: "color: var(--pm-color-text-primary);",
    expected: true,
  },
];

for (const regressionCase of scannerRegressionCases) {
  if (regressionCase.pattern.test(regressionCase.source) !== regressionCase.expected) {
    throw new Error(`color policy regression: ${regressionCase.description}`);
  }
}

async function sourceFiles(directory, files = []) {
  const entries = await readdir(directory, { withFileTypes: true });
  for (const entry of entries) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isSymbolicLink()) {
      throw new Error(`color policy refuses source symlink ${entryPath}`);
    }
    if (entry.isDirectory()) {
      await sourceFiles(entryPath, files);
    } else if (entry.isFile() && sourceExtensions.has(path.extname(entry.name))) {
      files.push(entryPath);
    }
  }
  return files;
}

function lineNumber(source, offset) {
  return source.slice(0, offset).split(/\r?\n/u).length;
}

const findings = [];
for (const root of sourceRoots) {
  for (const file of await sourceFiles(root)) {
    const source = await readFile(file, "utf8");
    const relative = path.relative(repositoryRoot, file);
    for (const candidate of literalPatterns) {
      const match = candidate.pattern.exec(source);
      if (match) {
        findings.push(
          `${relative}:${lineNumber(source, match.index)} contains a ${candidate.label}`,
        );
      }
    }

    if (generatedClassPattern.test(source)) {
      findings.push(
        `${relative} hardcodes a generated token class instead of using the typed token accessors`,
      );
    }
    if (rawColorVariablePattern.test(source)) {
      findings.push(
        `${relative} bypasses typed token access with a raw generated color variable`,
      );
    }
    if (
      /\.(?:ts|tsx)$/u.test(file) &&
      /\b(?:backgroundColor|borderColor|color|fill|outlineColor|stroke)\s*:/u.test(
        source,
      )
    ) {
      findings.push(
        `${relative} declares an inline color-bearing style instead of a typed token class`,
      );
    }
  }
}

if (findings.length > 0) {
  throw new Error(
    `hand-written UI color policy failed:\n  ${findings.join("\n  ")}`,
  );
}

process.stdout.write(
  "hand-written UI color policy: no literals, generated-class bypasses, or raw color-variable access\n",
);
