import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';

export interface ThemeState {
  logo?: string;
  title?: string;
}

export function useExternalTheme(): ThemeState {
  const [theme, setTheme] = useState<ThemeState>({});

  const reload = async () => {
    try {
      const res = await invoke<{
        css?: string;
        logo_base64?: string;
        title?: string;
      }>('load_external_theme');
      if (!res) return;

      // 1. Inject or update CSS
      let styleEl = document.getElementById('ccbaga-external-theme') as HTMLStyleElement | null;
      if (res.css) {
        if (!styleEl) {
          styleEl = document.createElement('style');
          styleEl.id = 'ccbaga-external-theme';
          document.head.appendChild(styleEl);
        }
        if (styleEl.textContent !== res.css) {
          styleEl.textContent = res.css;
        }
      } else if (styleEl) {
        styleEl.remove();
      }

      setTheme({
        logo: res.logo_base64,
        title: res.title,
      });
    } catch {
      // In mock/preview browser environment without backend, silently fallback
    }
  };

  useEffect(() => {
    void reload();
    window.addEventListener('focus', reload);
    return () => window.removeEventListener('focus', reload);
  }, []);

  return theme;
}
