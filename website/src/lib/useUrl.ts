import { useSession } from 'next-auth/react';
import { demoNpmPackages, demoRepos } from '../demoIcons';

export type IconType = 'github' | 'npm';

export function useUrl(type: IconType, slug: string, all = false) {
  const { data } = useSession();

  slug = slug.toLowerCase();

  const includeToken = !(type === 'npm' ? demoNpmPackages : demoRepos).find(
    (packageSlug) => packageSlug.toLowerCase() === slug
  );

  // if it starts with a reserved name, then prefix with
  // an @ symbol
  if (slug.startsWith('npm/')) {
    slug = `@${slug}`;
  }

  const url = `https://github-icons.com/${
    type !== 'github' ? `${type}/` : ''
  }${slug.toLowerCase()}${all ? '/all' : ''}${
    includeToken && data?.accessToken ? `?token=${data.accessToken}` : ''
  }`;

  return url;
}
