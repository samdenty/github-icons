import '../styles/globals.css';
import { AppProps } from 'next/app';
import { SessionProvider } from 'next-auth/react';
import Head from 'next/head';
import { BackgroundEffect } from '../components/BackgroundEffect';
import { Session } from 'next-auth';
import { RelayEnvironmentProvider } from 'react-relay/hooks';
import { getInitialPreloadedQuery, getRelayProps } from 'relay-nextjs/app';
import { getClientEnvironment } from '../lib/clientEnvironment';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { persistQueryClient } from '@tanstack/react-query-persist-client';
import { createSyncStoragePersister } from '@tanstack/query-sync-storage-persister';
import React from 'react';
import { Analytics } from '@vercel/analytics/react';

if (typeof window === 'undefined') {
  React.useLayoutEffect = React.useEffect;
}

const clientEnv = getClientEnvironment();
const initialPreloadedQuery = getInitialPreloadedQuery({
  createClientEnvironment: () => getClientEnvironment()!,
});

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      cacheTime: 1000 * 60 * 60 * 24, // 24 hours
      suspense: true,
    },
  },
});

if (typeof window !== 'undefined') {
  const localStoragePersister = createSyncStoragePersister({
    storage: window.localStorage,
  });

  persistQueryClient({
    queryClient,
    persister: localStoragePersister,
  });
}

export default function App({
  Component,
  pageProps: { session, ...pageProps },
}: AppProps<{ session?: Session }>) {
  const relayProps = getRelayProps(pageProps, initialPreloadedQuery);
  const env = relayProps.preloadedQuery?.environment ?? clientEnv!;

  return (
    <QueryClientProvider client={queryClient}>
      <SessionProvider session={session}>
        <RelayEnvironmentProvider environment={env}>
          <Head>
            <title>GitHub / NPM Icons</title>
            <meta
              name="description"
              content="Website for searching scraped GitHub and NPM icons, along with a JSON / Image API"
            />
            <link rel="icon" href="/favicon.ico" />
          </Head>

          <BackgroundEffect />
          <Component {...pageProps} {...relayProps} />
          <Analytics />
        </RelayEnvironmentProvider>
      </SessionProvider>
    </QueryClientProvider>
  );
}
