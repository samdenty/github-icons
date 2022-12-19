import styled from '@emotion/styled';
import Head from 'next/head';
import { useQuery } from 'react-query';
import { useUrl } from '../../lib/useUrl';
import { Icon } from './Icon';
import _ from 'lodash';

export interface RepoProps {
  slug: string;
}

const SSR = typeof window === 'undefined';

const AddIcon = styled.button`
  border-radius: 16px;
  padding: 10px;
  border: 3px solid #0000006e;
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
  slug = slug.toLowerCase();

  if (SSR) {
    return null;
  }

  const {
    default: ReactJson,
  }: typeof import('react-json-view') = require('react-json-view');

  const url = useUrl(slug, true);
  const { data } = useQuery<Icon[]>(
    [slug, 'all'],
    () => fetch(url).then((res) => res.json()),
    { cacheTime: 0 }
  );

  const iconByKinds = _.groupBy(data, 'kind');

  return (
    <>
      <Head>
        <title>{slug} - github-icons</title>
      </Head>

      <div>
        {data && (
          <>
            {Object.entries(iconByKinds).map(
              ([kind, icons], iconByKindIndex) => (
                <div>
                  {kind}
                  {icons.map((icon, index) => (
                    <Icon
                      key={JSON.stringify(icon)}
                      {...icon}
                      selected={iconByKindIndex === 0 && index === 0}
                    />
                  ))}
                </div>
              )
            )}
            <ReactJson src={data} name={false} theme="summerfruit" />
          </>
        )}
      </div>
    </>
  );
}
