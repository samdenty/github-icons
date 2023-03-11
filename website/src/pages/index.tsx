import { useSession, signIn, signOut } from 'next-auth/react';
import dynamic from 'next/dynamic';
import { Suspense } from 'react';
import _ from 'lodash';
import styled from '@emotion/styled';
import { demoNpmPackages } from '../demoIcons';
import { useQuery } from '../lib/useQuery';
import Link from 'next/link';
import { CgProfile } from 'react-icons/cg';

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

const StyledUserRepos = styled.div`
  width: 250px;
  margin-left: 50px;
  margin-top: 50px;

  > * {
    grid-template-columns: repeat(1, 1fr) !important;
  }
`;

const Header = styled.header`
  display: flex;
  justify-content: flex-end;
`;

const Main = styled.main`
  display: flex;
  padding: 0 50px;
`;

const Content = styled.div`
  display: flex;
  flex-grow: 1;
  flex-direction: column;
  align-items: center;
`;

const SearchProfile = styled(Link)`
  border: none;
  display: flex;
  justify-content: center;
  align-items: center;
  background: #23282c;
  height: 50px;
  border-radius: 6px;
  margin-bottom: 20px;
  transition: opacity 0.1s ease;

  &:hover {
    background: #2f363c;
  }

  > * {
    margin-right: 10px;
    height: 24px;
    width: 24px;
  }
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
          </>
        ) : (
          <>
            Not signed in <br />
            <button onClick={() => signIn('github')}>Sign in</button>
          </>
        )}
      </Header>

      <Main>
        <Content>
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
        </Content>

        {session && (
          <StyledUserRepos>
            <SearchProfile href={`/${session.user.id}`}>
              <CgProfile />
              Go to @{session.user.id}'s icons
            </SearchProfile>

            <Suspense fallback="loading">
              <UserRepos user={session.user.id} />
            </Suspense>
          </StyledUserRepos>
        )}
      </Main>
    </>
  );
}
