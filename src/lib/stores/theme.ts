import { browser } from "$app/environment";

type Theme = "light" | "dark";

function getSystemTheme(): Theme {
  if (browser && window.matchMedia("(prefers-color-scheme: dark)").matches) {
    return "dark";
  }
  return "light";
}

function getSavedTheme(): Theme {
  if (!browser) return "light";
  const saved = localStorage.getItem("loka-theme");
  if (saved === "light" || saved === "dark") return saved;
  return getSystemTheme();
}

function applyTheme(theme: Theme) {
  if (!browser) return;
  document.documentElement.setAttribute("data-theme", theme);
  localStorage.setItem("loka-theme", theme);
}

let currentTheme: Theme = "light";

/** Call once on app mount (client only) to read saved preference and apply it. */
export function initTheme(): void {
  currentTheme = getSavedTheme();
  applyTheme(currentTheme);
}

export function getTheme(): Theme {
  return currentTheme;
}

export function toggleTheme(): Theme {
  currentTheme = currentTheme === "light" ? "dark" : "light";
  applyTheme(currentTheme);
  return currentTheme;
}

export function setTheme(theme: Theme): void {
  currentTheme = theme;
  applyTheme(currentTheme);
}
