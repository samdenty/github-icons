import { useEffect } from 'react';
import { useOctokit, gql } from '../hooks/useOctokit';

const PinnedReposQuery = gql`
  query {
    viewer {
      pinnedItems(first: 6, types: REPOSITORY) {
        nodes {
          ... on Repository {
            name
          }
        }
      }
    }
  }
`;

export function UserRepos() {
  const octokit = useOctokit();

  useEffect(() => {
    octokit?.graphql(PinnedReposQuery).then(console.log);
  }, [octokit]);

  return null;
}
