import '../styles/globals.css';
import { AppProps } from 'next/app';
import { SessionProvider } from 'next-auth/react';
import Head from 'next/head';
import { BackgroundEffect } from '../components/BackgroundEffect';

export default function App({
  Component,
  pageProps: { session, ...pageProps },
}: AppProps) {
  return (
    <SessionProvider session={session}>
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
    </SessionProvider>
  );
}
