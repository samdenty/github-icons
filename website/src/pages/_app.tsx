import '../styles/globals.css';
import { AppProps } from 'next/app';
import { SessionProvider } from 'next-auth/react';
import Head from 'next/head';
import { BackgroundEffect } from '../components/BackgroundEffect';
import { Session } from 'next-auth';
import { RelayEnvironmentProvider } from 'react-relay/hooks';
import { getInitialPreloadedQuery, getRelayProps } from 'relay-nextjs/app';
import { getClientEnvironment } from '../lib/clientEnvironment';
import { QueryClient, QueryClientProvider } from 'react-query';

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
            <title>github-icons</title>
            <meta
              name="description"
              content="Chrome Extension, API & Mac App/CLI that adds icons to your repos"
            />
            <link rel="icon" href="/favicon.ico" />
          </Head>

          <BackgroundEffect />
          <Component {...pageProps} {...relayProps} />
        </RelayEnvironmentProvider>
      </SessionProvider>
    </QueryClientProvider>
  );
}
