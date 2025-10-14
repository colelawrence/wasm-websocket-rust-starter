import type { Adaptor, Router, Observable } from "./types";
import type { ShortestPathParams, PathResult } from "../../dist-types";
import { enhanceObservable } from "./enhancedObservable";

export interface CreateRouterOptions {
  adaptor: Adaptor;
}

export function createRouter(options: CreateRouterOptions): Router {
  const { adaptor } = options;
  let requestId = 0;
  let initialized = false;

  // Lazy initialization wrapper
  async function ensureInitialized() {
    if (!initialized) {
      await adaptor.init();
      initialized = true;
    }
  }

  return {
    find_shortest_path(params: ShortestPathParams) {
      const obs: Observable<PathResult> = {
        subscribe: (observer) => {
          ensureInitialized()
            .then(() => {
              requestId++;
              const request: import("./types").RequestEnum = {
                Call: [requestId, { find_shortest_path: params }],
              };

              adaptor.sendRequest(request).subscribe({
                next: (response) => {
                  const [, responseEnum] = response;
                  if ("N" in responseEnum && "find_shortest_path" in responseEnum.N) {
                    observer.next?.(responseEnum.N.find_shortest_path);
                  }
                },
                ...(observer.error && { error: observer.error }),
                ...(observer.complete && { complete: observer.complete }),
              });
            })
            .catch((error) => {
              observer.error?.(
                error instanceof Error ? error.message : String(error),
              );
            });
        },
      };
      return enhanceObservable(obs);
    },

    dispose() {
      adaptor.dispose?.();
    },
  };
}
