import { useEffect, useState } from 'react';
import { Monitor, Moon, Palette, Sun, X } from 'lucide-react';
import { useTheme } from 'next-themes';
import { Button } from '@/components/ui/button';
import {
  DOCS_THEME_ACCENTS,
  DOCS_THEME_STORAGE_KEYS,
  applyDocsThemeAccent,
  resolveDocsCustomAccent,
  resolveDocsThemeAccent,
  type DocsThemeMode,
} from '@/lib/theme';

const themeModes = [
  { value: 'system', label: 'System', icon: Monitor },
  { value: 'light', label: 'Light', icon: Sun },
  { value: 'dark', label: 'Dark', icon: Moon },
] as const;

export default function DocsThemeCustomizer() {
  const { resolvedTheme, theme, setTheme } = useTheme();
  const [isOpen, setIsOpen] = useState(false);
  const [accent, setAccent] = useState(() =>
    resolveDocsThemeAccent(localStorage.getItem(DOCS_THEME_STORAGE_KEYS.accent)),
  );
  const [customAccent, setCustomAccentState] = useState(() =>
    resolveDocsCustomAccent(localStorage.getItem(DOCS_THEME_STORAGE_KEYS.customAccent)),
  );

  useEffect(() => {
    const mode: DocsThemeMode = resolvedTheme === 'dark' ? 'dark' : 'light';
    applyDocsThemeAccent(accent, mode, customAccent);
  }, [accent, customAccent, resolvedTheme]);

  const setPresetAccent = (value: keyof typeof DOCS_THEME_ACCENTS) => {
    setAccent(value);
    localStorage.setItem(DOCS_THEME_STORAGE_KEYS.accent, value);
  };

  const setCustomAccent = (value: `#${string}`) => {
    setCustomAccentState(value);
    setAccent('custom');
    localStorage.setItem(DOCS_THEME_STORAGE_KEYS.customAccent, value);
    localStorage.setItem(DOCS_THEME_STORAGE_KEYS.accent, 'custom');
  };

  return (
    <div className="fixed bottom-5 right-5 z-50">
      {isOpen ? (
        <div className="w-[320px] rounded-2xl border border-border/70 bg-background/95 p-4 shadow-2xl backdrop-blur">
          <div className="mb-4 flex items-start justify-between gap-3">
            <div>
              <p className="text-sm font-semibold text-foreground">Theme Settings</p>
              <p className="text-xs text-muted-foreground">Switch mode and accent color for the docs.</p>
            </div>
            <Button variant="ghost" size="icon-sm" onClick={() => setIsOpen(false)} aria-label="Close theme settings">
              <X />
            </Button>
          </div>

          <div className="mb-4 grid grid-cols-3 gap-2">
            {themeModes.map(({ value, label, icon: Icon }) => (
              <button
                key={value}
                type="button"
                onClick={() => setTheme(value)}
                className={`rounded-xl border px-3 py-2 text-left transition-colors ${
                  theme === value
                    ? 'border-primary bg-primary text-primary-foreground'
                    : 'border-border bg-card hover:border-primary/40 hover:bg-accent'
                }`}
              >
                <Icon className="mb-2 size-4" />
                <span className="block text-xs font-medium">{label}</span>
              </button>
            ))}
          </div>

          <div className="grid grid-cols-3 gap-2">
            {Object.entries(DOCS_THEME_ACCENTS).map(([key, preset]) => (
              <button
                key={key}
                type="button"
                onClick={() => setPresetAccent(key as keyof typeof DOCS_THEME_ACCENTS)}
                className={`rounded-xl border p-2 transition-colors ${
                  accent === key
                    ? 'border-primary bg-primary/8'
                    : 'border-border bg-card hover:border-primary/40 hover:bg-accent'
                }`}
                title={preset.label}
              >
                <span
                  className="mb-2 block h-8 w-full rounded-lg"
                  style={{ backgroundColor: preset.swatch }}
                />
                <span className="block text-xs font-medium text-foreground">{preset.label}</span>
              </button>
            ))}

            <label
              className={`cursor-pointer rounded-xl border p-2 transition-colors ${
                accent === 'custom'
                  ? 'border-primary bg-primary/8'
                  : 'border-border bg-card hover:border-primary/40 hover:bg-accent'
              }`}
            >
              <span
                className="mb-2 flex h-8 w-full items-center justify-center rounded-lg"
                style={{ backgroundColor: customAccent }}
              >
                <Palette className="size-4 text-white" />
              </span>
              <span className="block text-xs font-medium text-foreground">Custom</span>
              <span className="block text-[10px] uppercase tracking-wide text-muted-foreground">
                {customAccent}
              </span>
              <input
                type="color"
                value={customAccent}
                onChange={(event) => setCustomAccent(event.target.value as `#${string}`)}
                className="sr-only"
                aria-label="Choose a custom accent color"
              />
            </label>
          </div>
        </div>
      ) : (
        <Button
          type="button"
          size="icon-lg"
          className="rounded-full shadow-xl"
          onClick={() => setIsOpen(true)}
          aria-label="Open theme settings"
        >
          <Palette />
        </Button>
      )}
    </div>
  );
}
