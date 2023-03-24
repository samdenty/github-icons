import styled from '@emotion/styled';
import Head from 'next/head';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { IconType, useUrl } from '../../lib/useUrl';
import { Icon } from './Icon';
import _ from 'lodash';
import { TfiReload } from 'react-icons/tfi';
import React from 'react';

interface IconsResponse {
  icons: Icon[] | null;
  errors: string[] | null;
}

const PRETTY_KINDS: Record<Icon['kind'], string> = {
  icon_field: `Root package.json "icon" field`,
  app_icon: `App Icons from repo's homepage`,
  site_favicon: `Favicons from repo's homepage`,
  site_logo: `Auto-detected logo on repo's homepage`,
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
  type: IconType;
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

const Main = styled.main``;

export function AllIcons({ type, slug }: RepoProps) {
  slug = slug.toLowerCase();

  if (SSR) {
    return null;
  }

  const {
    default: ReactJson,
  }: typeof import('react-json-view') = require('react-json-view');

  const allUrl = useUrl(type, slug, true);

  const refetchRef = React.useRef(false);
  const [isRefetching, setIsRefetching] = React.useState(false);

  const { data, refetch: _refetch } = useQuery<IconsResponse>(
    [slug, 'all'],
    () => {
      const url = new URL(allUrl);

      if (refetchRef.current) {
        refetchRef.current = false;
        url.searchParams.set('refetch', '1');
      }

      return fetch(url, { cache: 'reload' }).then((res) => res.json());
    },
    {
      cacheTime: 0,
      refetchOnWindowFocus: false,
    }
  );

  function refetch() {
    setIsRefetching(true);
    refetchRef.current = true;
    _refetch().finally(() => {
      setIsRefetching(false);
    });
  }

  const iconByKinds = _.groupBy(data!.icons, 'kind');

  return (
    <Main>
      <Head>
        <title>
          {slug} - {type === 'github' ? 'GitHub' : 'NPM'} Icons
        </title>
      </Head>

      <button onClick={refetch}>
        Refetch icons <TfiReload />
      </button>

      <div style={{ opacity: isRefetching ? 0.5 : 1 }}>
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
    </Main>
  );
}
