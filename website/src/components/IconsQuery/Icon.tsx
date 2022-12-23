import styled from '@emotion/styled';
import {
  IconButton,
  IconButtonBadge,
  IconButtonIcon,
  IconButtonLoading,
} from '../IconButton/IconButton';
import useFitText from 'use-fit-text';
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

const Slug = styled.div`
  display: flex;
  flex-direction: column;
  flex-grow: 1;
  margin-top: 5px;
  white-space: nowrap;
  word-break: break-all;
  overflow: hidden;
  width: 100%;
  font-size: 13px;
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

  let { fontSize, ref } = useFitText();

  if (parseInt(fontSize) < 80) {
    fontSize = undefined!;
  }

  return (
    <StyledIconButton type={type} slug={slug}>
      <Slug>
        <Name
          ref={ref}
          style={{
            fontSize,
            whiteSpace: fontSize ? 'pre' : 'break-spaces',
          }}
        >
          <StyledIconButtonBadge />
          {packageName}
        </Name>
        {org && <Owner>{org}/</Owner>}
      </Slug>
    </StyledIconButton>
  );
}
