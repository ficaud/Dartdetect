function normalizeBaseUrl(baseUrl) {
  return baseUrl.replace(/\/$/, '');
}

function inferDefaultBaseUrl() {
  if (typeof window === 'undefined') {
    return 'http://127.0.0.1:8080';
  }

  const protocol = window.location.protocol === 'https:' ? 'https:' : 'http:';
  return `${protocol}//${window.location.hostname}:8080`;
}

function toWebSocketUrl(httpBaseUrl) {
  if (httpBaseUrl.startsWith('https://')) {
    return `wss://${httpBaseUrl.slice('https://'.length)}`;
  }

  if (httpBaseUrl.startsWith('http://')) {
    return `ws://${httpBaseUrl.slice('http://'.length)}`;
  }

  return httpBaseUrl;
}

async function readResponse(response) {
  const contentType = response.headers.get('content-type') || '';
  const payload = contentType.includes('application/json')
    ? await response.json()
    : await response.text();

  if (!response.ok) {
    throw new Error(typeof payload === 'string' ? payload : JSON.stringify(payload));
  }

  return payload;
}

export function createServerProvider(options = {}) {
  const configuredBaseUrl =
    options.baseUrl || import.meta.env.VITE_SERVER_BASE_URL || inferDefaultBaseUrl();
  const httpBaseUrl = normalizeBaseUrl(configuredBaseUrl);
  const wsBaseUrl = toWebSocketUrl(httpBaseUrl);

  async function request(path, init) {
    const response = await fetch(`${httpBaseUrl}${path}`, init);
    return readResponse(response);
  }

  return {
    mode: 'server',

    async getLatestScore() {
      return request('/api/latest');
    },

    async subscribe(handlers) {
      const socket = new WebSocket(`${wsBaseUrl}/ws/impacts`);

      socket.addEventListener('message', (event) => {
        try {
          const parsed = JSON.parse(event.data);

          if (parsed.type === 'score' && handlers.onScore) {
            handlers.onScore(parsed.payload);
          }

          if (parsed.type === 'error' && handlers.onError) {
            handlers.onError(String(parsed.payload));
          }

          if (parsed.type === 'status' && handlers.onStatus) {
            handlers.onStatus(parsed.payload);
          }
        } catch (error) {
          if (handlers.onError) {
            handlers.onError(String(error));
          }
        }
      });

      socket.addEventListener('error', () => {
        if (handlers.onError) {
          handlers.onError('WebSocket connection error');
        }
      });

      return () => {
        if (socket.readyState === WebSocket.OPEN || socket.readyState === WebSocket.CONNECTING) {
          socket.close();
        }
      };
    },

    async getVersion() {
      return request('/version');
    },
  };
}
