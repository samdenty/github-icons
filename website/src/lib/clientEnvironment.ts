import { getSession } from 'next-auth/react';
import { hydrateRelayEnvironment } from 'relay-nextjs';
import { withHydrateDatetime } from 'relay-nextjs/date';
import { Environment, Network, Store, RecordSource } from 'relay-runtime';

export function createClientNetwork() {
  return Network.create(async (params, variables) => {
    const session = await getSession();
    if (!session) {
      throw new Error('session required!');
    }

    const response = await fetch('https://api.github.com/graphql', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${session!.accessToken}`,
      },
      body: JSON.stringify({
        query: params.text,
        variables,
      }),
    });

    const json = await response.text();
    return JSON.parse(json, withHydrateDatetime);
  });
}

let clientEnv: Environment | undefined;
export function getClientEnvironment() {
  if (typeof window === 'undefined') return null;

  if (clientEnv == null) {
    clientEnv = new Environment({
      network: createClientNetwork(),
      store: new Store(new RecordSource()),
      isServer: false,
    });

    hydrateRelayEnvironment(clientEnv);
  }

  return clientEnv;
}
