type ThemeVariables = Record<string, string>;

export type DocsThemeMode = 'light' | 'dark';
export type DocsThemeAccent =
  | 'amber'
  | 'blue'
  | 'emerald'
  | 'rose'
  | 'violet'
  | 'custom';

export type PresetDocsThemeAccent = Exclude<DocsThemeAccent, 'custom'>;

type AccentVariables = {
  '--theme-primary': string;
  '--theme-primary-foreground': string;
  '--theme-brand': string;
  '--theme-brand-foreground': string;
  '--theme-brand-secondary': string;
  '--theme-brand-secondary-foreground': string;
  '--theme-brand-200': string;
  '--theme-fd-primary': string;
  '--theme-fd-brand': string;
  '--theme-fd-primary-foreground': string;
  '--theme-sidebar-primary': string;
  '--theme-sidebar-primary-foreground': string;
};

type AccentPreset = {
  label: string;
  swatch: string;
  light: AccentVariables;
  dark: AccentVariables;
};

function toDeclarations(variables: ThemeVariables): string {
  return Object.entries(variables)
    .map(([name, value]) => `  ${name}: ${value};`)
    .join('\n');
}

function normalizeHexColor(value: string | null): `#${string}` | null {
  if (!value) {
    return null;
  }

  const hex = value.trim();
  return /^#[0-9a-fA-F]{6}$/.test(hex) ? (hex.toLowerCase() as `#${string}`) : null;
}

function hexToRgb(value: `#${string}`) {
  return {
    r: Number.parseInt(value.slice(1, 3), 16),
    g: Number.parseInt(value.slice(3, 5), 16),
    b: Number.parseInt(value.slice(5, 7), 16),
  };
}

function rgbToHex({ r, g, b }: { r: number; g: number; b: number }): `#${string}` {
  const toHex = (channel: number) => channel.toString(16).padStart(2, '0');
  return `#${toHex(r)}${toHex(g)}${toHex(b)}` as `#${string}`;
}

function mixHexColors(colorA: `#${string}`, colorB: `#${string}`, weight: number): `#${string}` {
  const a = hexToRgb(colorA);
  const b = hexToRgb(colorB);
  const mix = (from: number, to: number) => Math.round(from * (1 - weight) + to * weight);

  return rgbToHex({
    r: mix(a.r, b.r),
    g: mix(a.g, b.g),
    b: mix(a.b, b.b),
  });
}

function getContrastColor(color: `#${string}`): '#111111' | '#ffffff' {
  const { r, g, b } = hexToRgb(color);
  const channel = (input: number) => {
    const normalized = input / 255;
    return normalized <= 0.03928
      ? normalized / 12.92
      : ((normalized + 0.055) / 1.055) ** 2.4;
  };

  const luminance = 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
  return luminance > 0.45 ? '#111111' : '#ffffff';
}

function createAccentVariables(
  primary: `#${string}`,
  options: {
    foreground?: '#111111' | '#ffffff';
    secondary?: `#${string}`;
    secondaryForeground?: string;
    brand200?: `#${string}`;
    sidebarPrimary?: `#${string}`;
    sidebarPrimaryForeground?: string;
  } = {},
): AccentVariables {
  const primaryForeground = options.foreground ?? getContrastColor(primary);
  const brandSecondary = options.secondary ?? mixHexColors(primary, '#ffffff', 0.35);
  const brandSecondaryForeground = options.secondaryForeground ?? getContrastColor(brandSecondary);
  const brand200 = options.brand200 ?? mixHexColors(primary, '#ffffff', 0.72);
  const sidebarPrimary = options.sidebarPrimary ?? primary;
  const sidebarPrimaryForeground = options.sidebarPrimaryForeground ?? primaryForeground;

  return {
    '--theme-primary': primary,
    '--theme-primary-foreground': primaryForeground,
    '--theme-brand': primary,
    '--theme-brand-foreground': primaryForeground,
    '--theme-brand-secondary': brandSecondary,
    '--theme-brand-secondary-foreground': brandSecondaryForeground,
    '--theme-brand-200': brand200,
    '--theme-fd-primary': primary,
    '--theme-fd-brand': primary,
    '--theme-fd-primary-foreground': primaryForeground,
    '--theme-sidebar-primary': sidebarPrimary,
    '--theme-sidebar-primary-foreground': sidebarPrimaryForeground,
  };
}

const shared: ThemeVariables = {
  '--theme-font-sans': '"Inter", "Segoe UI", sans-serif',
  '--theme-font-mono': '"SFMono-Regular", "Consolas", monospace',
  '--theme-font-serif': '"Iowan Old Style", "Palatino Linotype", "Book Antiqua", serif',
  '--theme-radius': '0.625rem',
};

const lightBase: ThemeVariables = {
  '--theme-background': 'hsl(0, 0%, 96%)',
  '--theme-foreground': 'hsl(0, 0%, 3.9%)',
  '--theme-card': 'hsl(0, 0%, 94.7%)',
  '--theme-card-foreground': 'hsl(0, 0%, 3.9%)',
  '--theme-popover': 'hsl(0, 0%, 98%)',
  '--theme-popover-foreground': 'hsl(0, 0%, 15.1%)',
  '--theme-secondary': 'hsl(0, 0%, 93.1%)',
  '--theme-secondary-foreground': 'hsl(0, 0%, 9%)',
  '--theme-muted': 'hsl(0, 0%, 96.1%)',
  '--theme-muted-foreground': 'hsl(0, 0%, 45.1%)',
  '--theme-accent': 'hsla(0, 0%, 82%, 50%)',
  '--theme-accent-foreground': 'hsl(0, 0%, 9%)',
  '--theme-destructive': 'oklch(0.577 0.245 27.325)',
  '--theme-border': 'hsla(0, 0%, 80%, 50%)',
  '--theme-input': 'hsla(0, 0%, 80%, 50%)',
  '--theme-ring': 'hsl(0, 0%, 63.9%)',
  '--theme-fd-background': 'hsl(0, 0%, 96%)',
  '--theme-fd-foreground': 'hsl(0, 0%, 3.9%)',
  '--theme-fd-muted': 'hsl(0, 0%, 96.1%)',
  '--theme-fd-muted-foreground': 'hsl(0, 0%, 45.1%)',
  '--theme-fd-popover': 'hsl(0, 0%, 98%)',
  '--theme-fd-popover-foreground': 'hsl(0, 0%, 15.1%)',
  '--theme-fd-card': 'hsl(0, 0%, 94.7%)',
  '--theme-fd-card-foreground': 'hsl(0, 0%, 3.9%)',
  '--theme-fd-border': 'hsla(0, 0%, 80%, 50%)',
  '--theme-fd-secondary': 'hsl(0, 0%, 93.1%)',
  '--theme-fd-secondary-foreground': 'hsl(0, 0%, 9%)',
  '--theme-fd-accent': 'hsla(0, 0%, 82%, 50%)',
  '--theme-fd-accent-foreground': 'hsl(0, 0%, 9%)',
  '--theme-fd-ring': 'hsl(0, 0%, 63.9%)',
  '--theme-chart-1': 'oklch(0.85 0.13 165)',
  '--theme-chart-2': 'oklch(0.77 0.15 163)',
  '--theme-chart-3': 'oklch(0.7 0.15 162)',
  '--theme-chart-4': 'oklch(0.6 0.13 163)',
  '--theme-chart-5': 'oklch(0.51 0.1 166)',
  '--theme-sidebar': 'hsl(0, 0%, 98%)',
  '--theme-sidebar-foreground': 'hsl(0, 0%, 15.1%)',
  '--theme-sidebar-accent': 'hsla(0, 0%, 82%, 50%)',
  '--theme-sidebar-accent-foreground': 'hsl(0, 0%, 9%)',
  '--theme-sidebar-border': 'hsla(0, 0%, 80%, 50%)',
  '--theme-sidebar-ring': 'hsl(0, 0%, 63.9%)',
  '--theme-sidebar-muted': 'hsl(0, 0%, 96.1%)',
  '--theme-sidebar-muted-foreground': 'hsl(0, 0%, 45.1%)',
  '--theme-subnav-background': 'hsl(0 0% 96% / 80%)',
};

const darkBase: ThemeVariables = {
  '--theme-background': 'hsl(0, 0%, 7.04%)',
  '--theme-foreground': 'hsl(0, 0%, 92%)',
  '--theme-card': 'hsl(0, 0%, 9.8%)',
  '--theme-card-foreground': 'hsl(0, 0%, 98%)',
  '--theme-popover': 'hsl(0, 0%, 11.6%)',
  '--theme-popover-foreground': 'hsl(0, 0%, 86.9%)',
  '--theme-secondary': 'hsl(0, 0%, 12.9%)',
  '--theme-secondary-foreground': 'hsl(0, 0%, 92%)',
  '--theme-muted': 'hsl(0, 0%, 12.9%)',
  '--theme-muted-foreground': 'hsla(0, 0%, 70%, 0.8)',
  '--theme-accent': 'hsla(0, 0%, 40.9%, 30%)',
  '--theme-accent-foreground': 'hsl(0, 0%, 90%)',
  '--theme-destructive': 'oklch(0.704 0.191 22.216)',
  '--theme-border': 'hsla(0, 0%, 40%, 20%)',
  '--theme-input': 'hsla(0, 0%, 40%, 20%)',
  '--theme-ring': 'hsl(0, 0%, 54.9%)',
  '--theme-fd-background': 'hsl(0, 0%, 7.04%)',
  '--theme-fd-foreground': 'hsl(0, 0%, 92%)',
  '--theme-fd-muted': 'hsl(0, 0%, 12.9%)',
  '--theme-fd-muted-foreground': 'hsla(0, 0%, 70%, 0.8)',
  '--theme-fd-popover': 'hsl(0, 0%, 11.6%)',
  '--theme-fd-popover-foreground': 'hsl(0, 0%, 86.9%)',
  '--theme-fd-card': 'hsl(0, 0%, 9.8%)',
  '--theme-fd-card-foreground': 'hsl(0, 0%, 98%)',
  '--theme-fd-border': 'hsla(0, 0%, 40%, 20%)',
  '--theme-fd-secondary': 'hsl(0, 0%, 12.9%)',
  '--theme-fd-secondary-foreground': 'hsl(0, 0%, 92%)',
  '--theme-fd-accent': 'hsla(0, 0%, 40.9%, 30%)',
  '--theme-fd-accent-foreground': 'hsl(0, 0%, 90%)',
  '--theme-fd-ring': 'hsl(0, 0%, 54.9%)',
  '--theme-chart-1': 'oklch(0.85 0.13 165)',
  '--theme-chart-2': 'oklch(0.77 0.15 163)',
  '--theme-chart-3': 'oklch(0.7 0.15 162)',
  '--theme-chart-4': 'oklch(0.6 0.13 163)',
  '--theme-chart-5': 'oklch(0.51 0.1 166)',
  '--theme-sidebar': 'oklch(0.21 0.034 264.665)',
  '--theme-sidebar-foreground': 'oklch(0.985 0.002 247.839)',
  '--theme-sidebar-accent': 'oklch(0.7 0.15 162)',
  '--theme-sidebar-accent-foreground': 'oklch(0.26 0.05 173)',
  '--theme-sidebar-border': 'oklch(1 0 0 / 10%)',
  '--theme-sidebar-ring': 'oklch(0.551 0.027 264.364)',
  '--theme-sidebar-muted': 'hsl(0, 0%, 16%)',
  '--theme-sidebar-muted-foreground': 'hsl(0, 0%, 72%)',
  '--theme-subnav-background': 'hsl(0 0% 7.04% / 80%)',
};

export const DOCS_THEME_STORAGE_KEYS = {
  accent: 'docs-theme-accent',
  customAccent: 'docs-theme-custom-accent',
} as const;

export const DEFAULT_DOCS_THEME_ACCENT: DocsThemeAccent = 'custom';
export const DEFAULT_DOCS_CUSTOM_ACCENT = '#64748b';

export const DOCS_THEME_ACCENTS: Record<PresetDocsThemeAccent, AccentPreset> = {
  amber: {
    label: 'Amber',
    swatch: '#f59e0b',
    light: createAccentVariables('#cc6f23', {
      secondary: '#c6bb58',
      secondaryForeground: '#97890c',
      brand200: '#ff9f00',
    }),
    dark: createAccentVariables('#fff383', {
      foreground: '#111111',
      secondary: '#fc7744',
      secondaryForeground: '#521700',
      brand200: '#fff7c8',
      sidebarPrimary: '#7fe0bc',
      sidebarPrimaryForeground: '#173528',
    }),
  },
  blue: {
    label: 'Blue',
    swatch: '#3b82f6',
    light: createAccentVariables('#2563eb'),
    dark: createAccentVariables('#60a5fa', { foreground: '#111111' }),
  },
  emerald: {
    label: 'Emerald',
    swatch: '#10b981',
    light: createAccentVariables('#059669'),
    dark: createAccentVariables('#34d399', { foreground: '#111111' }),
  },
  rose: {
    label: 'Rose',
    swatch: '#f43f5e',
    light: createAccentVariables('#e11d48'),
    dark: createAccentVariables('#fb7185', { foreground: '#111111' }),
  },
  violet: {
    label: 'Violet',
    swatch: '#8b5cf6',
    light: createAccentVariables('#7c3aed'),
    dark: createAccentVariables('#a78bfa', { foreground: '#111111' }),
  },
};

function createCustomAccent(mode: DocsThemeMode, color: `#${string}`): AccentVariables {
  const primary = mode === 'dark' ? mixHexColors(color, '#ffffff', 0.12) : color;
  const secondary =
    mode === 'dark'
      ? mixHexColors(color, '#111111', 0.45)
      : mixHexColors(color, '#ffffff', 0.35);
  const brand200 =
    mode === 'dark'
      ? mixHexColors(color, '#ffffff', 0.72)
      : mixHexColors(color, '#ffffff', 0.75);

  return createAccentVariables(primary, {
    secondary,
    secondaryForeground: getContrastColor(secondary),
    brand200,
  });
}

function getAccentVariables(accent: DocsThemeAccent, mode: DocsThemeMode, customAccent: `#${string}`) {
  if (accent === 'custom') {
    return createCustomAccent(mode, customAccent);
  }

  return DOCS_THEME_ACCENTS[accent][mode];
}

export function resolveDocsThemeAccent(value: string | null): DocsThemeAccent {
  return value === 'custom' || Boolean(value && value in DOCS_THEME_ACCENTS)
    ? (value as DocsThemeAccent)
    : DEFAULT_DOCS_THEME_ACCENT;
}

export function resolveDocsCustomAccent(value: string | null): `#${string}` {
  return normalizeHexColor(value) ?? DEFAULT_DOCS_CUSTOM_ACCENT;
}

export function applyDocsThemeAccent(
  accent: DocsThemeAccent,
  mode: DocsThemeMode,
  customAccent: `#${string}` = DEFAULT_DOCS_CUSTOM_ACCENT,
) {
  const variables = getAccentVariables(accent, mode, customAccent);
  const root = document.documentElement.style;

  Object.entries(variables).forEach(([name, value]) => {
    root.setProperty(name, value);
  });
}

const customLight = createCustomAccent('light', DEFAULT_DOCS_CUSTOM_ACCENT as `#${string}`);
const customDark = createCustomAccent('dark', DEFAULT_DOCS_CUSTOM_ACCENT as `#${string}`);

const defaultLight = {
  ...shared,
  ...lightBase,
  ...(DEFAULT_DOCS_THEME_ACCENT === 'custom'
    ? customLight
    : DOCS_THEME_ACCENTS[DEFAULT_DOCS_THEME_ACCENT as PresetDocsThemeAccent].light),
};
const defaultDark = {
  ...shared,
  ...darkBase,
  ...(DEFAULT_DOCS_THEME_ACCENT === 'custom'
    ? customDark
    : DOCS_THEME_ACCENTS[DEFAULT_DOCS_THEME_ACCENT as PresetDocsThemeAccent].dark),
};

export const docsThemeStyles = `
:root {
  color-scheme: light;
${toDeclarations(defaultLight)}
}

.light {
  color-scheme: light;
${toDeclarations(defaultLight)}
}

.dark {
  color-scheme: dark;
${toDeclarations(defaultDark)}
}

@media (prefers-color-scheme: dark) {
  :root:not(.light):not(.dark) {
    color-scheme: dark;
${toDeclarations(defaultDark)}
  }
}

.dark #nd-sidebar {
  --color-fd-muted: var(--theme-sidebar-muted);
  --color-fd-secondary: var(--theme-secondary);
  --color-fd-muted-foreground: var(--theme-sidebar-muted-foreground);
}

.dark #nd-subnav {
  --color-fd-background: var(--theme-subnav-background);
}
`.trim();
