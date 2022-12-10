import { RepoButton } from '../components/RepoButton';
import { Repo } from '../components/Repo';
import demoRepos from '../../demo-repos.json';
import styled from '@emotion/styled';
import { useSession, signIn, signOut } from 'next-auth/react';
import { useRouter } from 'next/router';
import Modal from 'react-modal';
import { useContextualRouting } from 'next-use-contextual-routing';
import dynamic from 'next/dynamic';
import { Suspense } from 'react';
import useFitText from 'use-fit-text';

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

const StyledRepoButton = styled(RepoButton)`
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

const RepoName = styled.div`
  ${StyledRepoButton}:hover & {
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
        <Repo slug={`${router.query.owner}/${router.query.repo}`}></Repo>
      </Modal>

      <main>
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

        <Repos>
          {demoRepos.map((slug) => {
            const [owner, repo] = slug.split('/');
            const { fontSize, ref } = useFitText();

            return (
              <StyledRepoButton key={slug} slug={slug}>
                <Slug>
                  <Owner>{owner}/</Owner>
                  <RepoName ref={ref} style={{ fontSize }}>
                    {repo}
                  </RepoName>
                </Slug>
              </StyledRepoButton>
            );
          })}
        </Repos>
      </main>
    </>
  );
}
