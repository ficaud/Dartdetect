import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export function createDesktopProvider() {
  return {
    mode: 'desktop',

    async getStatus() {
      return { running: false };
    },

    async getLatestScore() {
      return null;
    },

    async subscribe(handlers) {
      const unlisteners = [];

      if (handlers.onScore) {
        unlisteners.push(
          await listen('simulation:score', (event) => {
            handlers.onScore(event.payload);
          })
        );
      }

      if (handlers.onError) {
        unlisteners.push(
          await listen('simulation:error', (event) => {
            handlers.onError(String(event.payload));
          })
        );
      }

      if (handlers.onStatus) {
        unlisteners.push(
          await listen('simulation:status', (event) => {
            handlers.onStatus(event.payload);
          })
        );
      }

      return () => {
        for (const unlisten of unlisteners) {
          unlisten();
        }
      };
    }
  };
}
