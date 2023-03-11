import { UserRepos_pinnedReposQuery } from '../queries/__generated__/UserRepos_pinnedReposQuery.graphql';
import { graphql, useLazyLoadQuery } from 'react-relay';
import { IconButton } from './IconButton/IconButton';
import styled from '@emotion/styled';
import { AiOutlineStar } from 'react-icons/ai';
import { GoRepoForked } from 'react-icons/go';

const PinnedReposQuery = graphql`
  query UserRepos_pinnedReposQuery($user: String!) {
    repositoryOwner(login: $user) {
      ... on ProfileOwner {
        pinnedItems(first: 6, types: [REPOSITORY]) {
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
  }
`;

const StyledUserRepos = styled.div<{ full: boolean }>`
  display: grid;
  grid-template-columns: repeat(${(props) => (props.full ? 3 : 6)}, 1fr);
  grid-gap: 8px;
  margin-bottom: 25px;

  @media (max-width: 900px) {
    grid-template-columns: repeat(3, 1fr);
  }

  @media (max-width: 600px) {
    grid-template-columns: repeat(2, 1fr);
  }
`;

const StyledRepoButton = styled(IconButton)`
  --size: 54px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  padding: 8px 16px;
  font-size: 12px;
`;

const Content = styled.div`
  display: flex;
  flex-direction: column;
  margin-left: 20px;
  height: 100%;

  > *:not(:last-child) {
    margin-bottom: 8px;
  }
`;

const Name = styled.div``;

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
`;

const Description = styled.p`
  flex-grow: 1;
`;

const Info = styled.div`
  display: flex;

  > * {
    display: flex;
    align-items: center;

    > :first-of-type {
      margin-right: 4px;
    }

    &:not(:last-child) {
      margin-right: 16px;
    }
  }
`;

const Language = styled.div``;

const LanguageColor = styled.div<{ color?: string }>`
  border-radius: 50%;
  width: 12px;
  height: 12px;
  background: ${(props) => props.color};
  display: ${(props) => (props.color ? 'block' : 'none')};
`;

const Stars = styled.div``;

const Forks = styled.div``;

export interface UserReposProps {
  user: string;
  full?: boolean;
}

export default function UserRepos({ user, full = false }: UserReposProps) {
  const query = useLazyLoadQuery<UserRepos_pinnedReposQuery>(PinnedReposQuery, {
    user,
  });

  if (!query.repositoryOwner) {
    return <>User not found</>;
  }

  if (!query.repositoryOwner.pinnedItems?.nodes?.length) {
    return null;
  }

  return (
    <StyledUserRepos full={full}>
      {query.repositoryOwner.pinnedItems.nodes.map((repo) => (
        <StyledRepoButton
          key={repo!.nameWithOwner}
          type="github"
          slug={repo!.nameWithOwner!}
        >
          <Content>
            <Name>
              <Slug>{repo!.nameWithOwner!}</Slug>
              {full && <Badge>Public</Badge>}
            </Name>
            {full && repo!.description && (
              <Description>{repo!.description}</Description>
            )}
            <Info>
              {repo!.primaryLanguage && (
                <Language>
                  <LanguageColor color={repo!.primaryLanguage.color!} />
                  {repo!.primaryLanguage.name}
                </Language>
              )}

              {repo!.stargazers && (
                <Stars>
                  <AiOutlineStar />
                  {repo!.stargazers.totalCount}
                </Stars>
              )}

              {full && repo!.forkCount !== undefined && (
                <Forks>
                  <GoRepoForked />
                  {repo!.forkCount}
                </Forks>
              )}
            </Info>
          </Content>
        </StyledRepoButton>
      ))}
    </StyledUserRepos>
  );
}
