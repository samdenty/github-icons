import styled from '@emotion/styled';
import Head from 'next/head';
import { useQuery } from 'react-query';
import { useUrl } from '../../lib/useUrl';
import { Icon } from './Icon';
import _ from 'lodash';

const PRETTY_KINDS: Record<Icon['kind'], string> = {
  icon_field: `Root package.json "icon" field`,
  app_icon: `App Icons from repo's homepage`,
  site_favicon: `Favicons from repo's homepage`,
  site_logo: `Logo on repo's homepage`,
  repo_file: `Files within repo`,
  avatar: `Repo owner's Avatar`,
  framework_icon: `Framework Icon`,
  org_avatar: `Organization's Avatar`,
  user_avatar_fallback: `User's Avatar (fallback)`,
  readme_image: `Image at top of README`,
};

function prettyKind(kind: Icon['kind'], kindIndex: number) {
  return PRETTY_KINDS[kind];
}

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

const KindGroup = styled.div``;

const Kind = styled.div``;

const Icons = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, 100px);
  grid-gap: 8px;
`;

export function Repo({ slug }: RepoProps) {
  slug = slug.toLowerCase();

  if (SSR) {
    return null;
  }

  const {
    default: ReactJson,
  }: typeof import('react-json-view') = require('react-json-view');

  const url = useUrl('github', slug, true);
  const { data } = useQuery<Icon[]>(
    [slug, 'all'],
    () => fetch(url, { cache: 'reload' }).then((res) => res.json()),
    { cacheTime: 0 }
  );

  const iconByKinds = _.groupBy(data, 'kind');

  return (
    <>
      <Head>
        <title>{slug} - GitHub Icons</title>
      </Head>

      <div>
        {data && (
          <>
            <h2>Auto-Detected repo icons</h2>
            {Object.entries(iconByKinds).map(
              ([kind, icons], iconByKindIndex) => (
                <KindGroup>
                  <Kind>
                    {prettyKind(kind as Icon['kind'], iconByKindIndex)}:
                  </Kind>

                  <Icons>
                    {icons.map((icon, index) => (
                      <Icon
                        key={JSON.stringify(icon)}
                        {...icon}
                        selected={iconByKindIndex === 0 && index === 0}
                      />
                    ))}
                  </Icons>
                </KindGroup>
              )
            )}
            <ReactJson src={data} name={false} theme="summerfruit" />
          </>
        )}
      </div>
    </>
  );
}
