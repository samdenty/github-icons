import { Repo } from '../components/Repo/Repo';
import { useSession, signIn, signOut } from 'next-auth/react';
import { useRouter } from 'next/router';
import Modal from 'react-modal';
import { useContextualRouting } from 'next-use-contextual-routing';
import dynamic from 'next/dynamic';
import { Suspense, startTransition, useState } from 'react';
import _ from 'lodash';

const UserRepos = dynamic(() => import('../components/UserRepos'), {
  ssr: false,
});

const IconsQuery = dynamic(
  () => import('../components/IconsQuery/IconsQuery'),
  {
    ssr: false,
  }
);

Modal.setAppElement('#__next');

interface HomeProps {}

export default function Home({}: HomeProps) {
  const { data: session } = useSession();
  const router = useRouter();
  const { returnHref } = useContextualRouting();

  const [query, setQuery] = useState('');

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
        <input
          value={query}
          placeholder="Enter a NPM package / GitHub repo"
          onChange={(e) => {
            setQuery(e.target.value);
          }}
        ></input>

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

        <Suspense fallback="loading">
          <IconsQuery query={query} />
        </Suspense>
      </main>
    </>
  );
}
