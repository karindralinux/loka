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

// Apply on load (before first render)
let currentTheme: Theme = getSavedTheme();
if (browser) applyTheme(currentTheme);

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
