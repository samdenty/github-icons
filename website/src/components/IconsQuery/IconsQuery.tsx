import { useQuery } from '@tanstack/react-query';
import { demoIcons } from '../../demoIcons';
import { search } from './search';
import { useSession } from 'next-auth/react';
import { useDebounce } from 'use-debounce';
import IconsGrid, { Icon } from '../IconsGrid/IconsGrid';

export interface IconsQueryProps {
  query: string;
  strict?: boolean;
}

export default function IconsQuery({ query, strict = false }: IconsQueryProps) {
  [query] = useDebounce(query, 300);
  const session = useSession();

  const { data } = useQuery(
    ['search', query.toLowerCase(), strict, !!session.data?.accessToken],
    async (): Promise<Icon[]> => {
      if (!query) {
        return null!;
      }

      return search(query, { strict, githubToken: session.data?.accessToken });
    },
    {
      staleTime: /* 1 hour */ 1000 * 60 * 60,
    }
  );

  return <IconsGrid icons={!query ? demoIcons : data!} />;
}
