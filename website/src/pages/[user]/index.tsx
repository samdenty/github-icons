import { useRouter } from 'next/router';
import UserRepos from '../../components/UserRepos';
import { Suspense } from 'react';
import dynamic from 'next/dynamic';
import { useQuery } from '../../lib/useQuery';
import styled from '@emotion/styled';
import { useSession } from 'next-auth/react';

const Search = dynamic(() => import('../../components/Search'), {
  ssr: false,
});

const IconsQuery = dynamic(
  () => import('../../components/IconsQuery/IconsQuery'),
  {
    ssr: false,
  }
);

const Main = styled.div`
  display: flex;
  width: 100%;
  height: 100%;
  justify-content: center;
  padding: 0 50px;
`;

const Profile = styled.div`
  display: flex;
  flex-direction: column;
  justify-content: center;
  margin-right: 50px;
`;

const Avatar = styled.img`
  border-radius: 50%;
  height: 256px;
  width: 256px;
`;

const Content = styled.div`
  display: flex;
  flex-grow: 1;
  flex-direction: column;
  align-items: center;
  padding-top: 50px;
`;

export default function UserPage() {
  const router = useRouter();
  const { user } = router.query;
  const [query, setQuery] = useQuery();
  const { data: session } = useSession();

  if (typeof user !== 'string') {
    return null;
  }

  return (
    <Main>
      <Profile>
        <Avatar src={`https://github.com/${user}.png`} />
      </Profile>

      <Content>
        {session && <UserRepos user={user} full />}

        <Search
          query={query}
          onQuery={setQuery}
          placeholder={`Search @${user}'s GitHub repos icons`}
        />

        <Suspense fallback="loading">
          <IconsQuery query={`${user}/${query}`} strict />
        </Suspense>
      </Content>
    </Main>
  );
}
