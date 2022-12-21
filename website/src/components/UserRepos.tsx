import { UserRepos_pinnedReposQuery } from '../queries/__generated__/UserRepos_pinnedReposQuery.graphql';
import { graphql, useLazyLoadQuery } from 'react-relay';
import { IconButton } from './IconButton';
import styled from '@emotion/styled';

const PinnedReposQuery = graphql`
  query UserRepos_pinnedReposQuery {
    viewer {
      pinnedItems(first: 6, types: REPOSITORY) {
        nodes {
          ... on Repository {
            nameWithOwner
            isPrivate
            description
            primaryLanguage {
              name
              color
            }
            stargazers {
              totalCount
            }
            forkCount
          }
        }
      }
    }
  }
`;

const StyledUserRepos = styled.div`
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  grid-template-rows: repeat(3, 1fr);
  grid-gap: 8px;
`;

const StyledRepoButton = styled(IconButton)`
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  padding: 16px;

  img {
    height: 54px;
    width: 54px;
    margin-right: 20px;
  }
`;

const Content = styled.div`
  display: flex;
  flex-direction: column;

  > *:not(:last-child) {
    margin-bottom: 8px;
  }
`;

const Name = styled.div`
  display: flex;
`;

const Slug = styled.span`
  color: #58a6ff;
  margin-right: 8px;

  ${StyledRepoButton}:hover & {
    text-decoration: underline;
  }
`;

const Badge = styled.span`
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: #8b949e;
  font-size: 85%;
  padding: 0.12em 0.5em;
  border-radius: 2em;
  font-weight: 500;
  line-height: 18px;
`;

const Description = styled.p``;

const Info = styled.div`
  display: flex;

  > * {
    flex-grow: 1;

    &:not(:last-child) {
      margin-right: 16px;
    }
  }
`;

const Language = styled.div``;

const LanguageColor = styled.div`
  border-radius: 50%;
  width: 12px;
  height: 12px;
`;

const Stars = styled.div``;

const Forks = styled.div``;

export default function UserRepos({}) {
  const query = useLazyLoadQuery<UserRepos_pinnedReposQuery>(
    PinnedReposQuery,
    {}
  );

  return (
    <StyledUserRepos>
      {query.viewer.pinnedItems.nodes?.map((repo) => (
        <StyledRepoButton slug={repo!.nameWithOwner!}>
          <Content>
            <Name>
              <Slug>{repo!.nameWithOwner!}</Slug>
              <Badge>Public</Badge>
            </Name>
            {repo!.description && (
              <Description>{repo!.description}</Description>
            )}
            <Info>
              {repo!.primaryLanguage && (
                <Language>
                  <LanguageColor></LanguageColor>
                  {repo!.primaryLanguage.name}
                </Language>
              )}

              {repo!.stargazers && <Stars>{repo!.stargazers.totalCount}</Stars>}

              {repo!.forkCount !== undefined && (
                <Forks>{repo!.forkCount}</Forks>
              )}
            </Info>
          </Content>
        </StyledRepoButton>
      ))}
    </StyledUserRepos>
  );
}
