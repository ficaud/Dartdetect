import { createDesktopProvider } from './providers/desktopProvider';
import { createServerProvider } from './providers/serverProvider';

function hasTauriRuntime() {
  if (typeof window === 'undefined') {
    return false;
  }

  return Boolean(window.__TAURI__ || window.__TAURI_INTERNALS__);
}

export function resolveProvisioning() {
  return {
    requested: 'auto',
    mode: hasTauriRuntime() ? 'desktop' : 'server'
  };
}

export function createDataProvider(options = {}) {
  const { requested, mode } = resolveProvisioning();

  const provider =
    mode === 'desktop'
      ? createDesktopProvider()
      : createServerProvider({ baseUrl: options.serverBaseUrl });

  return {
    requested,
    mode,
    provider
  };
}
