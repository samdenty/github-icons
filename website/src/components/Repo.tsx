import Head from 'next/head';

export interface RepoProps {
  slug: string;
}

const SSR = typeof window === 'undefined';

export function Repo({ slug }: RepoProps) {
  return (
    <>
      {!SSR && (
        <Head>
          <title>{slug} - github-icons</title>
        </Head>
      )}
      <div>{slug}</div>
    </>
  );
}
