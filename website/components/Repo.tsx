import styled from '@emotion/styled';
import useFitText from 'use-fit-text';

export interface RepoProps {
  slug: string;
}

const StyledRepo = styled.div`
  display: flex;
  flex-direction: column;
  text-align: center;
  align-items: center;
  white-space: nowrap;
  overflow: hidden;
`;

const Logo = styled.img`
  height: 70px;
  width: 70px;
  object-fit: contain;
`;

const Owner = styled.div`
  margin-top: 5px;
  opacity: 0.5;
  font-size: 70%;
  line-height: 8px;
`;

const RepoName = styled.div`
  line-height: 14px;
`;

export function Repo({ slug }: RepoProps) {
  const { fontSize, ref } = useFitText();

  const [owner, repo] = slug.split('/');

  return (
    <StyledRepo ref={ref} style={{ fontSize }}>
      <Logo alt={slug} src={`https://github-icons.com/${slug}`} />
      <Owner>{owner}/</Owner>
      <RepoName>{repo}</RepoName>
    </StyledRepo>
  );
}
