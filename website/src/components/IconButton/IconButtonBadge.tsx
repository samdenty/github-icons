import styled from '@emotion/styled';
import { IconContext } from './IconButton';
import { useContext } from 'react';
import { IconType } from '../../lib/useUrl';

const StyledIconButtonBadge = styled.img`
  border-radius: 3px;
`;

const TYPE_ICONS: Record<IconType, string> = {
  npm: 'https://static.npmjs.com/1996fcfdf7ca81ea795f67f093d7f449.png',
  github: 'https://github.githubassets.com/favicons/favicon-dark.svg',
};

export function IconButtonBadge(
  props: Partial<React.HTMLAttributes<HTMLImageElement>>
) {
  const { type } = useContext(IconContext);

  return <StyledIconButtonBadge {...props} src={TYPE_ICONS[type]} />;
}
