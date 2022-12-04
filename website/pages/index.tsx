import { RepoButton } from '../components/RepoButton';
import { Repo } from '../components/Repo';
import demoRepos from '../demo-repos.json';
import styled from '@emotion/styled';
import { useSession, signIn, signOut } from 'next-auth/react';
import { useRouter } from 'next/router';
import Modal from 'react-modal';
import { useContextualRouting } from 'next-use-contextual-routing';

Modal.setAppElement('#__next');

const Repos = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(80px, 1fr));
  grid-gap: 18px;
  width: 100%;
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
          </>
        ) : (
          <>
            Not signed in <br />
            <button onClick={() => signIn('github')}>Sign in</button>
          </>
        )}
        <Repos>
          {demoRepos.map((slug) => (
            <RepoButton key={slug} slug={slug} />
          ))}
        </Repos>
      </main>
    </>
  );
}
