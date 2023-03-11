import styled from '@emotion/styled';
import { useSession } from 'next-auth/react';
import Link from 'next/link';
import { graphql, useLazyLoadQuery } from 'react-relay';
import { UserOrgs_orgsQuery } from '../queries/__generated__/UserOrgs_orgsQuery.graphql';
import { Suspense } from 'react';

const OrgsQuery = graphql`
  query UserOrgs_orgsQuery {
    viewer {
      organizations(first: 100) {
        nodes {
          login
        }
      }
    }
  }
`;

const StyledUserOrgs = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(50px, 1fr));
  grid-gap: 8px;
`;

const StyledOrganization = styled(Link)`
  display: flex;
  justify-content: center;
`;

const Avatar = styled.img`
  height: 50px;
  width: 50px;
  border-radius: 50%;
  transition: filter 0.2s ease;

  &:hover {
    filter: brightness(1.2) contrast(0.8);
  }
`;

interface OrganizationProps {
  org: string;
}

function Organization({ org }: OrganizationProps) {
  return (
    <StyledOrganization href={`/${org}`}>
      <Avatar src={`https://github.com/${org}.png`} />
    </StyledOrganization>
  );
}

function Organizations() {
  const query = useLazyLoadQuery<UserOrgs_orgsQuery>(OrgsQuery, {});

  return (
    <>
      {query.viewer.organizations.nodes?.map((node) => (
        <Organization key={node!.login} org={node!.login} />
      ))}
    </>
  );
}

export function UserOrgs() {
  const { data: session } = useSession();

  return (
    <StyledUserOrgs>
      <Organization org={session!.user.id} />

      <Suspense fallback={<></>}>
        <Organizations />
      </Suspense>
    </StyledUserOrgs>
  );
}
