import styled from '@emotion/styled';
import { useSession } from 'next-auth/react';
import { useContextualRouting } from 'next-use-contextual-routing';
import Link from 'next/link';
import { useUrl } from '../lib/useUrl';

export interface RepoButtonProps {
  slug: string;
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

export function IconButton({ slug, children, ...props }: RepoButtonProps) {
  const { makeContextualHref } = useContextualRouting();
  const iconUrl = useUrl(slug);

  const [owner, repo] = slug.split('/');

  return (
    <RepoLink
      {...props}
      href={makeContextualHref({ owner, repo })}
      as={`/${slug}`}
    >
      <Logo alt={slug} src={iconUrl} />
      {typeof children === 'function' ? children({ owner, repo }) : children}
    </RepoLink>
  );
}
