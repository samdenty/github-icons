import styled from '@emotion/styled';
import { useSession } from 'next-auth/react';
import Head from 'next/head';
import { useQuery } from 'react-query';
import { useUrl } from '../../lib/useUrl';
import { Icon } from './Icon';

export interface RepoProps {
  slug: string;
}

const SSR = typeof window === 'undefined';

const AddIcon = styled.button`
  border-radius: 16px;
  padding: 10px;
  border: 4px solid #0000006e;
  background: #00000021;
  font-size: 70px;
  line-height: 1px;
  aspect-ratio: 1;
  color: #0000006e;
  text-align: center;
  cursor: pointer;

  @media (prefers-color-scheme: dark) {
    border-color: #ffffff6e;
    background: #ffffff21;
    color: #ffffff6e;
  }

  &:hover {
    background: #00000075;
    color: #ffffffcc;

    @media (prefers-color-scheme: dark) {
      background: #ffffff75;
      color: #000000cc;
    }
  }
`;

export function Repo({ slug }: RepoProps) {
  const url = useUrl(slug, true);
  const { data } = useQuery<Icon[]>([slug, 'all'], () =>
    fetch(url).then((res) => res.json())
  );

  return (
    <>
      {!SSR && (
        <Head>
          <title>{slug} - github-icons</title>
        </Head>
      )}
      <div>
        {data?.map((icon, i) => (
          <Icon key={JSON.stringify(icon)} {...icon} selected={i === 0} />
        ))}
      </div>
    </>
  );
}
