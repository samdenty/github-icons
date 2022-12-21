import { IconButton } from '../components/IconButton';
import { Repo } from '../components/Repo/Repo';
import demo from '../../demo.json';
import styled from '@emotion/styled';
import { useSession, signIn, signOut } from 'next-auth/react';
import { useRouter } from 'next/router';
import Modal from 'react-modal';
import { useContextualRouting } from 'next-use-contextual-routing';
import dynamic from 'next/dynamic';
import { Suspense } from 'react';
import useFitText from 'use-fit-text';
import { Search } from '../components/Search';

const UserRepos = dynamic(() => import('../components/UserRepos'), {
  ssr: false,
});

Modal.setAppElement('#__next');

const Repos = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(80px, 1fr));
  grid-gap: 18px;
  width: 100%;
`;

const StyledIconButton = styled(IconButton)`
  flex-direction: column;
  text-align: center;
  width: 80px;

  &:hover img {
    transform: scale(1.1);
  }
`;

const Slug = styled.div`
  display: flex;
  flex-direction: column;
  justify-content: center;
  flex-grow: 1;
  margin-top: 5px;
  white-space: nowrap;
  overflow: hidden;
  width: 100%;
  font-size: 13px;
`;

const Owner = styled.div`
  opacity: 0.5;
  font-size: 77%;
`;

const Name = styled.div`
  ${StyledIconButton}:hover & {
    text-decoration: underline;
  }
`;

export default function Home() {
  const { data: session } = useSession();
  const router = useRouter();
  const { returnHref } = useContextualRouting();

  return (
    <>
      <Modal
        isOpen={!!router.query.repo}
        onRequestClose={() => router.push(returnHref)}
        style={{
          overlay: {
            backgroundColor: 'rgba(255, 255, 255, 0.3)',
          },
        }}
      >
        {router.query.repo && (
          <Suspense fallback="loading">
            <Repo slug={`${router.query.owner}/${router.query.repo}`}></Repo>
          </Suspense>
        )}
      </Modal>

      <main>
        <Search />
        {session ? (
          <>
            Signed in as {session.accessToken} <br />
            <button onClick={() => signOut()}>Sign out</button>
            <Suspense fallback="loading">
              <UserRepos />
            </Suspense>
          </>
        ) : (
          <>
            Not signed in <br />
            <button onClick={() => signIn('github')}>Sign in</button>
          </>
        )}
        NPM packages:
        <Repos>
          {demo.npmPackages.map((slug) => {
            let [org, packageName] = slug.split('/') as [
              string | undefined,
              string
            ];

            if (!packageName) {
              packageName = org!;
              org = undefined;
            }

            const { fontSize, ref } = useFitText();

            return (
              <StyledIconButton key={slug} type="npm" slug={slug}>
                <Slug>
                  {org && <Owner>{org}/</Owner>}
                  <Name ref={ref} style={{ fontSize }}>
                    {packageName}
                  </Name>
                </Slug>
              </StyledIconButton>
            );
          })}
        </Repos>
        GitHub repos:
        <Repos>
          {demo.repos.map((slug) => {
            const [owner, repo] = slug.split('/');
            const { fontSize, ref } = useFitText();

            return (
              <StyledIconButton key={slug} type="github" slug={slug}>
                <Slug>
                  <Owner>{owner}/</Owner>
                  <Name ref={ref} style={{ fontSize }}>
                    {repo}
                  </Name>
                </Slug>
              </StyledIconButton>
            );
          })}
        </Repos>
      </main>
    </>
  );
}
