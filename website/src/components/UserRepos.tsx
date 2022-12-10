import { UserRepos_pinnedReposQuery } from '../queries/__generated__/UserRepos_pinnedReposQuery.graphql';
import { graphql, useLazyLoadQuery } from 'react-relay';
import { RepoButton } from './RepoButton';

const PinnedReposQuery = graphql`
  query UserRepos_pinnedReposQuery {
    viewer {
      pinnedItems(first: 6, types: REPOSITORY) {
        nodes {
          ... on Repository {
            nameWithOwner
          }
        }
      }
    }
  }
`;

export default function UserRepos({}) {
  const query = useLazyLoadQuery<UserRepos_pinnedReposQuery>(
    PinnedReposQuery,
    {}
  );

  return (
    <div>
      {query.viewer.pinnedItems.nodes?.map((repo) => (
        <div>
          <RepoButton slug={repo!.nameWithOwner!}></RepoButton>
          {repo!.nameWithOwner!}
        </div>
      ))}
    </div>
  );
}
