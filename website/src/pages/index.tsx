import { useSession } from 'next-auth/react';
import dynamic from 'next/dynamic';
import { Suspense, useEffect, useState } from 'react';
import _ from 'lodash';
import styled from '@emotion/styled';
import { demoNpmPackages } from '../demoIcons';
import { useQuery } from '../lib/useQuery';
import { UserOrgs } from '../components/UserOrgs';
import copy from 'copy-to-clipboard';

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

const UsageSidebar = styled.div`
  display: flex;
  flex-direction: column;
  width: 250px;
  margin-right: 15px;
  align-items: center;
`;

const ProfileSidebar = styled.div`
  width: 250px;
  margin-left: 15px;
`;

const StyledUserRepos = styled(UserRepos)`
  grid-template-columns: repeat(1, 1fr) !important;
`;

const Main = styled.main`
  display: flex;
  padding: 50px 50px 0px 15px;
  height: 100%;
  width: 100%;
`;

const Content = styled.div`
  display: flex;
  flex-grow: 1;
  flex-direction: column;
  align-items: center;
`;

const Organizations = styled.div`
  background: #ffffff33;
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

const Code = styled.code`
  background: #ffffff33;
  color: #ffffff8c;
  font-size: 11px;
  border-radius: 6px;
  padding: 12px;
  margin-top: 12px;
  margin-bottom: 20px;
  width: 100%;
  word-break: break-all;
`;

const Token = styled(Code)`
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: orange;
`;

const Copy = styled.button``;

const ListIcons = styled.div`
  display: flex;
  flex-direction: column;
  opacity: 0.5;

  &:hover {
    opacity: 1;
  }
`;

interface HomeProps {}

export default function Home({}: HomeProps) {
  const { data: session } = useSession();
  const [query, setQuery] = useQuery();
  const [userToken, setUserToken] = useState<string | null>(null);

  useEffect(() => {
    if (session) {
      fetch(
        `https://github-icons.com/token-exchange?token=${session?.accessToken}`,
        {
          method: 'POST',
        }
      ).then(async (res) => {
        if (res.ok) {
          const token = await res.text();
          setUserToken(token);
        }
      });
    }
  }, [session?.accessToken]);

  return (
    <Main>
      {userToken && (
        <UsageSidebar>
          Your publicly shareable API token:
          <Token>
            {userToken}
            <Copy
              onClick={() => {
                copy(userToken);
              }}
            >
              Copy
            </Copy>
          </Token>
          <br />
          <span style={{ color: '#89da9b' }}>GitHub Repo Image API:</span>
          <Code>
            https://github-icons.com/
            <span style={{ color: '#3acc5a' }}>[user]</span>/
            <span style={{ color: '#3acc5a' }}>[repo]</span>?token=
            <span style={{ color: '#ab8d57' }}>{userToken}</span>
          </Code>
          <span style={{ color: '#e18a8a' }}>NPM Package Image API:</span>
          <Code>
            https://github-icons.com/
            <span style={{ color: '#e4e5e4' }}>npm</span>/
            <span style={{ color: '#ff5a5a' }}>[package]</span>?token=
            <span style={{ color: '#ab8d57' }}>{userToken}</span>
          </Code>
          <br />
          <ListIcons>
            List all icons for a repo:
            <Code>
              https://github-icons.com/
              <span style={{ color: '#3acc5a' }}>[user]</span>/
              <span style={{ color: '#3acc5a' }}>[repo]</span>/
              <span style={{ color: '#e4e5e4' }}>all</span>?token=
              <span style={{ color: '#ab8d57' }}>{userToken}</span>
            </Code>
            List all icons for a package:
            <Code>
              https://github-icons.com/
              <span style={{ color: '#e4e5e4' }}>npm</span>/
              <span style={{ color: '#ff5a5a' }}>[package]</span>/
              <span style={{ color: '#e4e5e4' }}>all</span>?token=
              <span style={{ color: '#ab8d57' }}>{userToken}</span>
            </Code>
          </ListIcons>
        </UsageSidebar>
      )}
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
          <Organizations>
            Click to show the icons for:
            <UserOrgs />
          </Organizations>

          <Suspense fallback="loading">
            <StyledUserRepos user={session.user.id} />
          </Suspense>
        </ProfileSidebar>
      )}
    </Main>
  );
}
