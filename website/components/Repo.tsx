import styled from '@emotion/styled';
import useFitText from 'use-fit-text';

export interface RepoProps {
  slug: string;
}

const StyledRepo = styled.a`
  display: flex;
  flex-direction: column;
  text-align: center;
  align-items: center;
  width: 80px;
  height: 115px;

  &:hover {
    cursor: pointer;
    img {
      opacity: 1;
      transform: scale(1.1);
      filter: brightness(1.1);
    }
  }
`;

const Logo = styled.img`
  height: 80px;
  width: 80px;
  object-fit: contain;
  border-radius: 10px;
  opacity: 0.8;
  transition: all 0.1s ease;
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

const RepoName = styled.div``;

export function Repo({ slug }: RepoProps) {
  const { fontSize, ref } = useFitText();
  const [owner, repo] = slug.split('/');

  const url = `https://github-icons.com/${slug}`;

  return (
    <StyledRepo href={`https://github.com/${slug}`} target="_blank">
      <Logo alt={slug} src={url} />
      <Slug>
        <Owner>{owner}/</Owner>
        <RepoName ref={ref} style={{ fontSize }}>
          {repo}
        </RepoName>
      </Slug>
    </StyledRepo>
  );
}
