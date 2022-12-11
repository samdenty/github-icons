import { useSession } from 'next-auth/react';

export function useUrl(slug: string, all = false) {
  const { data } = useSession();

  const url = `https://github-icons.com/${slug}${all ? '/all' : ''}${
    data?.accessToken ? `?token=${data.accessToken}` : ''
  }`;

  return url;
}
