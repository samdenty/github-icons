import styled from '@emotion/styled';
import { useContextualRouting } from 'next-use-contextual-routing';
import Link from 'next/link';
import { IconType, useUrl } from '../lib/useUrl';
import React from 'react';

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
  display: flex;
  align-items: center;

  &:hover {
    cursor: pointer;
  }
`;

const Logo = styled.img`
  height: 80px;
  width: 80px;
  object-fit: contain;
  border-radius: 10px;
  opacity: 0.8;
  transition: all 0.1s ease;

  ${RepoLink}:hover & {
    opacity: 1;
    filter: brightness(1.1);
  }
`;

export const IconButton = React.forwardRef(
  ({ slug, type, children, ...props }: IconButtonProps, ref) => {
    const { makeContextualHref } = useContextualRouting();
    const iconUrl = useUrl(type, slug);

    const [owner, repo] = slug.split('/');

    return (
      <RepoLink
        ref={ref}
        {...(props as any)}
        href={makeContextualHref({ owner, repo })}
        as={`/${type !== 'github' ? `${type}/` : ''}${slug}`}
      >
        <Logo alt={slug} src={iconUrl} />
        {typeof children === 'function' ? children({ owner, repo }) : children}
      </RepoLink>
    );
  }
);
