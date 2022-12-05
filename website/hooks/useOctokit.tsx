import { useSession } from 'next-auth/react';
import { Octokit } from 'octokit';
import * as React from 'react';

export const gql = String.raw;

const OctokitContext = React.createContext<Octokit | undefined>(undefined);

export const useOctokit = () => React.useContext(OctokitContext);

export function OctokitProvider(props: any) {
  const { data } = useSession();
  const octokit = React.useMemo(
    () => data && new Octokit({ auth: data.accessToken }),
    [data]
  );

  return <OctokitContext.Provider {...props} value={octokit} />;
}
