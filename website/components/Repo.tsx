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
`;

const Logo = styled.img`
  height: 70px;
  width: 70px;
  object-fit: contain;
`;

export function Repo({ slug }: RepoProps) {
  const { fontSize, ref } = useFitText();

  return (
    <StyledRepo ref={ref} style={{ fontSize }}>
      <Logo alt={slug} src={`https://github-icons.com/${slug}`} />
      {slug.split('/')[1]}
    </StyledRepo>
  );
}
