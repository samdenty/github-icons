import { IconType } from '../../lib/useUrl';
import styled from '@emotion/styled';
import { Icon } from './Icon';

export interface IconQuery {
  type: IconType;
  slug: string;
}

const StyledIconsGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(80px, 1fr));
  grid-gap: 18px;
  height: 100%;
  width: 100%;
  justify-items: center;
  overflow: scroll;
  padding: 35px;
  scroll-snap-type: y mandatory;
`;

export interface IconsGridProps {
  icons: IconQuery[];
}

export default function IconsGrid({ icons }: IconsGridProps) {
  return (
    <StyledIconsGrid>
      {icons.map((iconQuery) => (
        <Icon key={JSON.stringify(iconQuery)} {...iconQuery} />
      ))}
    </StyledIconsGrid>
  );
}
