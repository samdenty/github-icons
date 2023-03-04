import { useRouter } from 'next/router';
import UserRepos from '../../components/UserRepos';
import { Suspense, useState } from 'react';
import dynamic from 'next/dynamic';

const Search = dynamic(() => import('../../components/Search'), {
  ssr: false,
});

const IconsQuery = dynamic(
  () => import('../../components/IconsQuery/IconsQuery'),
  {
    ssr: false,
  }
);

export default function UserPage() {
  const router = useRouter();
  const { user } = router.query;
  const [query, setQuery] = useState('');

  if (typeof user !== 'string') {
    return null;
  }

  return (
    <>
      <UserRepos user={user} />

      <Search
        query={query}
        onQuery={setQuery}
        placeholder={`Search @${user}'s GitHub repos`}
      />

      <Suspense fallback="loading">
        <IconsQuery query={`${user}/${query}`} strict />
      </Suspense>
    </>
  );
}
