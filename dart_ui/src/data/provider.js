import { createServerProvider } from './providers/serverProvider';

function resolveProvisioning() {
  // Only server mode is active for now.
  // Future: detect Tauri or other runtimes here and return a different mode.
  return { mode: 'server' };
}

export function createDataProvider(options = {}) {
  const { mode } = resolveProvisioning();

  const provider = createServerProvider({ baseUrl: options.serverBaseUrl });

  return {
    mode,
    provider,
  };
}
