import styled from '@emotion/styled';
import {
  IconButton,
  IconButtonBadge,
  IconButtonIcon,
  IconButtonLoading,
} from '../IconButton/IconButton';
import { Textfit } from 'react-textfit';
import { IconType } from '../../lib/useUrl';

const StyledIconButton = styled(IconButton)`
  flex-direction: column;
  text-align: center;
  --size: 80px;

  ${IconButtonIcon} {
    transition: all 0.2s ease;
  }

  ${IconButtonLoading} {
    --size: 40px;
  }

  &:hover > ${IconButtonIcon} {
    transform: scale(1.1);
  }
`;

const StyledIconButtonBadge = styled(IconButtonBadge)`
  width: 1em;
  height: 1em;
  vertical-align: middle;
  margin-right: 4px;
`;

const Slug = styled(Textfit)`
  display: flex;
  flex-direction: column;
  flex-grow: 1;
  margin-top: 5px;
  white-space: break-spaces;
  word-break: break-all;
  overflow: hidden;
  width: 100%;
  min-height: 20px;
  max-height: 60px;
`;

const Owner = styled.div`
  opacity: 0.5;
  font-size: 77%;
`;

const Name = styled.div`
  ${StyledIconButton}:hover & {
    text-decoration: underline;
  }
`;

export interface IconProps {
  type: IconType;
  slug: string;
}

export function Icon({ type, slug }: IconProps) {
  let [org, packageName] = slug.split('/') as [string | undefined, string];

  if (!packageName) {
    packageName = org!;
    org = undefined;
  }

  return (
    <StyledIconButton type={type} slug={slug}>
      <Slug mode="multi" max={13}>
        <Name>
          <StyledIconButtonBadge />
          {packageName}
        </Name>
        {org && <Owner>{org}/</Owner>}
      </Slug>
    </StyledIconButton>
  );
}
