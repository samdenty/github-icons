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
    transform: scale(1.8);
    z-index: 10000;
    background: rgba(0, 0, 0, 0.8);
    border-radius: 20px;
    box-shadow: 0 0 8px 2px black;
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

  ${StyledIconButton}:hover & {
    transform: translateY(35px);
    z-index: 10000;
    font-size: 13px;
    background: rgba(0, 0, 0, 0.6);
    box-shadow: 0 0 8px 1px rgba(0, 0, 0, 0.8);
  }
`;

const Name = styled.div`
  ${StyledIconButton}:hover & {
    text-decoration: underline;
  }
`;

const Owner = styled.div`
  opacity: 0.5;
  font-size: 77%;
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
