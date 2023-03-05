import { useSession, signIn, signOut } from 'next-auth/react';
import { useRouter } from 'next/router';
import dynamic from 'next/dynamic';
import { Suspense } from 'react';
import _ from 'lodash';
import styled from '@emotion/styled';
import { demoNpmPackages } from '../demoIcons';
import { useQuery } from '../lib/useQuery';

const UserRepos = dynamic(() => import('../components/UserRepos'), {
  ssr: false,
});

const Search = dynamic(() => import('../components/Search'), {
  ssr: false,
});

const IconsQuery = dynamic(
  () => import('../components/IconsQuery/IconsQuery'),
  {
    ssr: false,
  }
);

const Header = styled.header`
  display: flex;
  justify-content: flex-end;
`;

const Main = styled.main`
  display: flex;
  flex-direction: column;
  align-items: center;
`;

interface HomeProps {}

export default function Home({}: HomeProps) {
  const { data: session } = useSession();
  const [query, setQuery] = useQuery();

  return (
    <>
      <Header>
        {session ? (
          <>
            Signed in as {session.accessToken} <br />
            <button onClick={() => signOut()}>Sign out</button>
            <Suspense fallback="loading">{/* <UserRepos /> */}</Suspense>
          </>
        ) : (
          <>
            Not signed in <br />
            <button onClick={() => signIn('github')}>Sign in</button>
          </>
        )}
      </Header>

      <Main>
        <Search
          query={query}
          onQuery={setQuery}
          placeholder={`Search for NPM packages and GitHub repos (i.e. ${demoNpmPackages
            .slice(0, 3)
            .join(', ')}...)`}
        />

        <Suspense fallback="loading">
          <IconsQuery query={query} />
        </Suspense>
      </Main>
    </>
  );
}
