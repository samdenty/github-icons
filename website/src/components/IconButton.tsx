import styled from '@emotion/styled';
import { useContextualRouting } from 'next-use-contextual-routing';
import Link from 'next/link';
import { IconType, useUrl } from '../lib/useUrl';
import React, { useState } from 'react';
import { PulseLoader } from 'react-spinners';

export interface IconButtonProps
  extends Omit<React.HTMLProps<HTMLAnchorElement>, 'children'> {
  slug: string;
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

const Loading = styled(PulseLoader)`
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

const Logo = styled.img`
  height: 100%;
  width: 100%;
  object-fit: contain;
  border-radius: 10px;
  opacity: 0.8;
  transition: all 0.2s ease;

  ${RepoLink}:hover & {
    opacity: 1;
    filter: brightness(1.1);
  }
`;

const Icon = styled.div<{ visible: boolean }>`
  position: relative;
  height: var(--size);
  width: var(--size);

  ${Loading} {
    opacity: ${(props) => (props.visible ? '0' : '1')};
  }

  ${Logo} {
    transform: scale(${(props) => (props.visible ? 1 : 0.1)});
    opacity: ${(props) => (props.visible ? '' : '0 !important')};
  }
`;

export const IconButton = React.forwardRef(
  ({ slug, type, children, ...props }: IconButtonProps, ref) => {
    const { makeContextualHref } = useContextualRouting();
    const iconUrl = useUrl(type, slug);
    const [visible, setVisible] = useState(false);

    const [owner, repo] = slug.split('/');

    return (
      <RepoLink
        ref={ref}
        {...(props as any)}
        href={makeContextualHref({ owner, repo })}
        as={`/${type !== 'github' ? `${type}/` : ''}${slug}`}
      >
        <Icon visible={visible}>
          <Loading />
          <Logo
            alt={slug}
            src={iconUrl}
            ref={(img) => {
              if (!img) {
                return;
              }

              let fallback = false;

              img.onerror = () => {
                if (fallback) {
                  return;
                }

                fallback = true;
                img.src = `https://static.npmjs.com/1996fcfdf7ca81ea795f67f093d7f449.png`;
              };

              img.onload = () => setVisible(true);

              if (img.complete) {
                setVisible(true);
              }
            }}
          />
        </Icon>
        {typeof children === 'function' ? children({ owner, repo }) : children}
      </RepoLink>
    );
  }
);
