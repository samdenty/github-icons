import { useSession } from 'next-auth/react';
import dynamic from 'next/dynamic';
import { Suspense } from 'react';
import _ from 'lodash';
import styled from '@emotion/styled';
import { demoNpmPackages } from '../demoIcons';
import { useQuery } from '../lib/useQuery';
import { UserOrgs } from '../components/UserOrgs';

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

const ProfileSidebar = styled.div`
  width: 250px;
  margin-left: 50px;
`;

const StyledUserRepos = styled(UserRepos)`
  grid-template-columns: repeat(1, 1fr) !important;
`;

const Main = styled.main`
  display: flex;
  padding: 50px 50px 0;
  height: 100%;
  width: 100%;
`;

const Content = styled.div`
  display: flex;
  flex-grow: 1;
  flex-direction: column;
  align-items: center;
`;

const Organzations = styled.div`
  background: #23282c;
  color: #ffffff8c;
  font-size: 12px;
  border-radius: 6px;
  padding: 12px;
  text-align: center;
  margin-bottom: 20px;

  > * {
    margin-top: 12px;
  }
`;

interface HomeProps {}

export default function Home({}: HomeProps) {
  const { data: session } = useSession();
  const [query, setQuery] = useQuery();

  return (
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
        <ProfileSidebar>
          <Organzations>
            Click to show the icons for:
            <UserOrgs />
          </Organzations>

          <Suspense fallback="loading">
            <StyledUserRepos user={session.user.id} />
          </Suspense>
        </ProfileSidebar>
      )}
    </Main>
  );
}
