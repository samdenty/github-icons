import '../styles/globals.css';
import { AppProps } from 'next/app';
import { SessionProvider } from 'next-auth/react';
import Head from 'next/head';
import { BackgroundEffect } from '../components/BackgroundEffect';
import { OctokitProvider } from '../hooks/useOctokit';
import { Session } from 'next-auth';

export default function App({
  Component,
  pageProps: { session, ...pageProps },
}: AppProps<{ session?: Session }>) {
  return (
    <SessionProvider session={session}>
      <OctokitProvider>
        <Head>
          <title>github-icons</title>
          <meta
            name="description"
            content="Chrome Extension, API & Mac App/CLI that adds icons to your repos"
          />
          <link rel="icon" href="/favicon.ico" />
        </Head>

        <BackgroundEffect />
        <Component {...pageProps} />
      </OctokitProvider>
    </SessionProvider>
  );
}
