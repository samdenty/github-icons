import { useQuery } from '@tanstack/react-query';
import { IconType } from '../../lib/useUrl';
import styled from '@emotion/styled';
import { Icon } from './Icon';
import { demoIcons } from '../../demoIcons';
import { search } from './search';
import { useSession } from 'next-auth/react';

export interface IconQuery {
  type: IconType;
  slug: string;
}

const StyledIconsQuery = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(80px, 1fr));
  grid-gap: 18px;
  width: 100%;
  place-items: center;
`;

export interface IconsQueryProps {
  query: string;
  strict?: boolean;
}

export default function IconsQuery({ query, strict = false }: IconsQueryProps) {
  const session = useSession();

  const { data } = useQuery(
    ['search', query, strict, !!session.data?.accessToken],
    async (): Promise<IconQuery[]> => {
      if (!query) {
        return demoIcons;
      }

      return search(query, { strict, githubToken: session.data?.accessToken });
    },
    {
      initialData: !query ? demoIcons : undefined,
      staleTime: !query ? 0 : /* 1 hour */ 1000 * 60 * 60,
    }
  );

  return (
    <StyledIconsQuery>
      {data!.map((iconQuery) => (
        <Icon key={JSON.stringify(iconQuery)} {...iconQuery} />
      ))}
    </StyledIconsQuery>
  );
}
