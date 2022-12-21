import { useSession } from 'next-auth/react';

export type IconType = 'github' | 'npm';

export function useUrl(type: IconType, slug: string, all = false) {
  const { data } = useSession();

  slug = slug.toLowerCase();

  // if it starts with a reserved name, then prefix with
  // an @ symbol
  if (slug.startsWith('npm/')) {
    slug = `@${slug}`;
  }

  const url = `https://github-icons.com/${
    type !== 'github' ? `${type}/` : ''
  }${slug.toLowerCase()}${all ? '/all' : ''}${
    data?.accessToken ? `?token=${data.accessToken}` : ''
  }`;

  return url;
}
