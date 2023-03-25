import { useSession } from 'next-auth/react';
import dynamic from 'next/dynamic';
import { Suspense, useEffect, useState } from 'react';
import _ from 'lodash';
import styled from '@emotion/styled';
import { demoNpmPackages } from '../demoIcons';
import { useQuery } from '../lib/useQuery';
import { UserOrgs } from '../components/UserOrgs';
import copy from 'copy-to-clipboard';
import { BsGithub } from 'react-icons/bs';
import { ImNpm } from 'react-icons/im';

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
  flex-shrink: 0;
  overflow-y: scroll;

  @media (max-width: 1100px) {
    display: none;
  }
`;

const ProfileSidebar = styled.div`
  width: 250px;
  margin-left: 15px;
  overflow-y: scroll;
  flex-shrink: 0;

  @media (max-width: 750px) {
    display: none;
  }
`;

const StyledUserRepos = styled(UserRepos)`
  grid-template-columns: repeat(1, 1fr) !important;
`;

const Main = styled.main`
  display: flex;
  padding: 50px 50px 0px 15px;
  height: 100%;
  width: 100%;

  @media (max-width: 750px) {
    padding-right: 15px;
  }
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
  font-size: 11.5px;
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
  align-items: center;
  opacity: 0.5;
  transition: opacity 0.2s ease;

  &:hover {
    opacity: 1;
  }
`;

const CodeTitle = styled.h5`
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  color: #ffffffc0;
  margin: 0;

  > * {
    height: 20px;
    width: 20px;
    margin-right: 5px;
  }
`;

const Star = styled.a`
  position: fixed;
  top: 15px;
  right: 50px;
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
      <Star href="https://github.com/samdenty/github-icons" target="_blank">
        <img
          alt="GitHub"
          src="https://img.shields.io/github/stars/samdenty/github-icons?style=social&label=Star"
        />
      </Star>
      {userToken ? (
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
          <CodeTitle>
            <BsGithub />
            GitHub Repo icon API:
          </CodeTitle>
          <Code>
            https://github-icons.com/
            <span style={{ color: '#3acc5a' }}>[user]</span>/
            <span style={{ color: '#3acc5a' }}>[repo]</span>?token=
            <span style={{ color: '#ab8d57' }}>{userToken}</span>
          </Code>
          <CodeTitle>
            <ImNpm />
            NPM Package icon API:
          </CodeTitle>
          <Code>
            https://github-icons.com/
            <span style={{ color: '#e4e5e4' }}>npm</span>/
            <span style={{ color: '#ff5a5a' }}>[package]</span>?token=
            <span style={{ color: '#ab8d57' }}>{userToken}</span>
          </Code>
          <ListIcons>
            <br />
            <CodeTitle>
              <BsGithub />
              List all icons for a repo:
            </CodeTitle>
            <Code>
              https://github-icons.com/
              <span style={{ color: '#3acc5a' }}>[user]</span>/
              <span style={{ color: '#3acc5a' }}>[repo]</span>/
              <span style={{ color: '#e4e5e4' }}>all</span>?token=
              <span style={{ color: '#ab8d57' }}>{userToken}</span>
            </Code>
            <CodeTitle>
              <ImNpm />
              List all icons for a package:
            </CodeTitle>
            <Code>
              https://github-icons.com/
              <span style={{ color: '#e4e5e4' }}>npm</span>/
              <span style={{ color: '#ff5a5a' }}>[package]</span>/
              <span style={{ color: '#e4e5e4' }}>all</span>?token=
              <span style={{ color: '#ab8d57' }}>{userToken}</span>
            </Code>
          </ListIcons>
        </UsageSidebar>
      ) : (
        session && <UsageSidebar>loading</UsageSidebar>
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
