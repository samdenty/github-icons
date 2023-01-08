import styled from '@emotion/styled';
import { demoNpmPackages } from '../demoIcons';

const StyledSearch = styled.input`
  background: #ffffff33;
  border: none;
  padding: 15px 10px;
  border-radius: 7px;
  backdrop-filter: blur(10px);
  width: 750px;
  text-align: center;
  outline: none;
  margin-bottom: 50px;

  @media (max-width: 850px) {
    width: 80%;
  }
`;

export interface SearchProps {
  onQuery(query: string): void;
  query: string;
}

export default function Search({ onQuery, query }: SearchProps) {
  return (
    <StyledSearch
      autoFocus
      placeholder={`Search for NPM packages and GitHub repos (i.e. ${demoNpmPackages
        .slice(0, 3)
        .join(', ')}...)`}
      value={query}
      onChange={(e) => {
        onQuery(e.target.value);
      }}
    ></StyledSearch>
  );
}
