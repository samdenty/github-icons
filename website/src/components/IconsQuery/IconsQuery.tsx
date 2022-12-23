import { QueryFunctionContext, useQuery } from 'react-query';
import { IconType } from '../../lib/useUrl';
import styled from '@emotion/styled';
import { Icon } from './Icon';
import { demoIcons } from '../../demoIcons';

export interface IconQuery {
  type: IconType;
  slug: string;
}

export interface IconsQueryProps {
  query: string;
}

const StyledIconsQuery = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(80px, 1fr));
  grid-gap: 18px;
  width: 100%;
`;

export default function IconsQuery({ query }: IconsQueryProps) {
  const { data } = useQuery(
    ['search', query],
    async (): Promise<IconQuery[]> => {
      if (!query) {
        return demoIcons;
      }

      const data = await fetch(
        `https://registry.npmjs.org/-/v1/search?text=${encodeURIComponent(
          query
        )}&size=40`
      ).then((res) => res.json());

      return data.objects.map((result: any) => ({
        type: 'npm',
        slug: result.package.name,
      }));
    },
    {
      initialData: !query ? demoIcons : undefined,
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
