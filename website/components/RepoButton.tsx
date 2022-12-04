import styled from '@emotion/styled';
import { useSession } from 'next-auth/react';
import { useContextualRouting } from 'next-use-contextual-routing';
import Link from 'next/link';
import useFitText from 'use-fit-text';

export interface RepoButtonProps {
  slug: string;
}

const RepoLink = styled(Link)`
  display: flex;
  flex-direction: column;
  text-align: center;
  align-items: center;
  width: 80px;
  height: 115px;

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
    transform: scale(1.1);
    filter: brightness(1.1);
  }
`;

const Slug = styled.div`
  display: flex;
  flex-direction: column;
  justify-content: center;
  flex-grow: 1;
  margin-top: 5px;
  white-space: nowrap;
  overflow: hidden;
  width: 100%;
  font-size: 13px;
`;

const Owner = styled.div`
  opacity: 0.5;
  font-size: 77%;
`;

const RepoName = styled.div`
  ${RepoLink}:hover & {
    text-decoration: underline;
  }
`;

export function RepoButton({ slug }: RepoButtonProps) {
  const { makeContextualHref } = useContextualRouting();
  const { data } = useSession();
  const { fontSize, ref } = useFitText();

  const [owner, repo] = slug.split('/');

  const url = `https://github-icons.com/${slug}${
    data?.accessToken ? `?token=${data.accessToken}` : ''
  }`;

  return (
    <RepoLink href={makeContextualHref({ owner, repo })} as={`/${slug}`}>
      <Logo alt={slug} src={url} />
      <Slug>
        <Owner>{owner}/</Owner>
        <RepoName ref={ref} style={{ fontSize }}>
          {repo}
        </RepoName>
      </Slug>
    </RepoLink>
  );
}
