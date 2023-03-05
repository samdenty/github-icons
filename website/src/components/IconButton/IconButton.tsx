import styled from '@emotion/styled';
import Link from 'next/link';
import { IconType, useUrl } from '../../lib/useUrl';
import React, { useState } from 'react';
import { PulseLoader } from 'react-spinners';
import { BsQuestionOctagonFill } from 'react-icons/bs';
import isPropValid from '@emotion/is-prop-valid';
import { IconButtonBadge } from './IconButtonBadge';

export interface IconButtonProps
  extends Omit<React.HTMLProps<HTMLAnchorElement>, 'children'> {
  slug: string;
  showBadge?: boolean;
  type: IconType;
  children?:
    | React.ReactNode
    | ((ownerAndRepo: { owner: string; repo: string }) => React.ReactNode);
  className?: string;
}

const RepoLink = styled(Link)`
  --size: 80px;
  display: flex;
  align-items: center;

  &:hover {
    cursor: pointer;
  }
`;

export const IconButtonLoading = styled(PulseLoader)`
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  white-space: pre;
  transition: all 0.5s ease;

  > * {
    background: rgba(255, 255, 255, 0.6) !important;
    width: calc(var(--size) * 0.1) !important;
    height: calc(var(--size) * 0.1) !important;
    margin: calc(var(--size) * 0.02) !important;
  }
`;

const Image = styled('img', { shouldForwardProp: isPropValid })<{
  showBadge?: boolean;
}>`
  height: var(--size);
  width: var(--size);
  object-fit: contain;
  border-radius: 10px;
  opacity: 0.8;
  transition: all 0.2s ease;
  filter: var(--border);

  ${RepoLink}:hover & {
    opacity: 1;
    filter: var(--border) brightness(1.1);
  }
`;

export const IconButtonIcon = styled('div', {
  shouldForwardProp: (prop) => prop !== 'loading',
})<{ loading: boolean }>`
  position: relative;
  height: var(--size);
  width: var(--size);

  ${IconButtonLoading} {
    opacity: ${(props) => (props.loading ? 1 : 0)};
  }

  ${Image} {
    transform: scale(${(props) => (props.loading ? 0.1 : 1)});
    opacity: ${(props) => (props.loading ? '0 !important' : '')};
  }
`;

export const IconContext = React.createContext<{ type: IconType }>(undefined!);

const StyledIconButtonBadge = styled(IconButtonBadge)`
  position: absolute;
  bottom: calc(var(--size) * -0.1) !important;
  right: calc(var(--size) * -0.1) !important;
  width: calc(var(--size) * 0.35) !important;
  height: calc(var(--size) * 0.35) !important;
`;

enum IconState {
  LOADING,
  VISIBLE,
  NO_ICON,
}

export const IconButton = React.forwardRef(
  ({ slug, showBadge, type, children, ...props }: IconButtonProps, ref) => {
    const iconUrl = useUrl(type, slug);
    const [state, setState] = useState<IconState>(IconState.LOADING);
    const [pixelated, setPixelated] = useState(false);

    const [owner, repo] = slug.split('/');

    return (
      <IconContext.Provider value={{ type }}>
        <RepoLink
          ref={ref}
          {...(props as any)}
          href={`/${type !== 'github' ? `${type}/` : ''}${slug}`}
        >
          <IconButtonIcon loading={state === IconState.LOADING}>
            <Image
              alt={slug}
              loading="lazy"
              as={state === IconState.NO_ICON ? BsQuestionOctagonFill : 'img'}
              showBadge={showBadge}
              src={iconUrl}
              style={{ imageRendering: pixelated ? 'pixelated' : undefined }}
              ref={
                state === IconState.NO_ICON
                  ? undefined
                  : (img) => {
                      if (!(img instanceof HTMLImageElement)) {
                        return;
                      }

                      img.onerror = () => {
                        setState(IconState.NO_ICON);
                      };

                      img.onload = () => {
                        setPixelated(
                          img.naturalHeight < img.height &&
                            img.naturalWidth < img.width
                        );
                        setState(IconState.VISIBLE);
                      };

                      if (img.complete) {
                        setState(IconState.VISIBLE);
                      }
                    }
              }
            />
            {showBadge && <StyledIconButtonBadge />}
            <IconButtonLoading />
          </IconButtonIcon>

          {typeof children === 'function'
            ? children({ owner, repo })
            : children}
        </RepoLink>
      </IconContext.Provider>
    );
  }
);

export * from './IconButtonBadge';
