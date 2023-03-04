import { fetchEventSource } from '@microsoft/fetch-event-source';
import { IconQuery } from './IconsQuery';

interface NPMSResults {
  results: {
    package: {
      name: string;
    };
  }[];
}

interface NPMResults {
  objects: {
    package: {
      name: string;
    };
  }[];
}

interface SourceGraphResult {
  repository: string;
  repoStars: number;
}

interface GithubResults {
  items: { full_name: string }[];
}

export async function search(
  query: string,
  { limit = 60, strict }: { limit?: number; strict?: boolean }
): Promise<IconQuery[]> {
  if (query.startsWith('@')) {
    return searchNPM(query, limit);
  }

  if (query.includes('/')) {
    return searchGithub(query, limit, strict);
  }

  const npmResults = await searchNPM(query, limit / 2);
  const githubResults = await searchGithub(query, limit / 2);

  const length =
    npmResults.length > githubResults.length
      ? npmResults.length
      : githubResults.length;

  const results = [];

  for (let i = 0; i < length; i++) {
    const githubResult = githubResults[i];
    if (githubResult) {
      results.push(githubResult);
    }

    const npmResult = npmResults[i];
    if (npmResult) {
      results.push(npmResult);
    }
  }

  return results;
}

async function searchNPM(query: string, limit: number): Promise<IconQuery[]> {
  try {
    return await racePromises([
      fetch(
        `https://registry.npmjs.org/-/v1/search?text=${encodeURIComponent(
          query
        )}&size=${Math.round(limit)}`
      )
        .then((res) => res.json())
        .then(({ objects }: NPMResults) => {
          return objects.map((object) => ({
            type: 'npm' as const,
            slug: object.package.name,
          }));
        }),
      fetch(
        `https://api.npms.io/v2/search?q=${encodeURIComponent(
          query
        )}&size=${Math.round(limit)}`
      )
        .then((res) => res.json())
        .then(({ results }: NPMSResults) => {
          return results.map((result) => ({
            type: 'npm' as const,
            slug: result.package.name,
          }));
        }),
    ]);
  } catch (e) {
    return [];
  }
}

async function searchGithub(
  query: string,
  limit: number,
  strict = false
): Promise<IconQuery[]> {
  const [, user, usersRepoQuery] = /^([^\/]+)\/(.*)/.exec(query) || [];

  async function searchGithubAPI() {
    const results: GithubResults = await fetch(
      `https://api.github.com/search/repositories?q=${encodeURIComponent(
        query + (strict && user ? ` user:${user}` : '')
      )}&per_page=${Math.round(limit)}`
    ).then((res) => res.json());

    return results.items.map((item) => item.full_name);
  }

  async function searchSourcegraph() {
    const sourcegraphQuery = [
      'repo:github.com',
      `repo:${
        strict && user ? `github.com/${user}/.*${usersRepoQuery}` : query
      }`,
      'fork:yes',
      'archived:yes',
      'select:repo',
      'timeout:2s',
    ].join(' ');

    const results = await new Promise<SourceGraphResult[]>((resolve) =>
      fetchEventSource(
        `https://sourcegraph.com/.api/search/stream?display=${Math.round(
          limit
        )}&q=${encodeURIComponent(sourcegraphQuery)}`,
        {
          onmessage({ data, event }) {
            if (event === 'matches') {
              resolve(JSON.parse(data));
            } else if (event === 'done') {
              resolve([]);
            }
          },
        }
      )
    );

    return results
      .map(({ repository }) => {
        const [, slug] =
          /^github\.com\/([^\/]+\/[^\/]+)$/.exec(repository) ?? [];
        return slug;
      })
      .filter(Boolean);
  }

  const [github, sourcegraph] = await Promise.all([
    searchGithubAPI().catch(() => []),
    searchSourcegraph().catch(() => []),
  ]);

  return [
    ...new Set(
      [...new Array(limit)]
        .map((_, i) => [sourcegraph[i], github[i]])
        .flat()
        .filter(Boolean)
    ),
  ].map((slug) => ({ type: 'github' as const, slug }));
}

function racePromises<T>(promises: Promise<T>[]) {
  let count = 0;

  return new Promise<T>((resolve, reject) => {
    for (const promise of promises) {
      promise.then(resolve, (err) => {
        count++;

        if (count === promises.length) {
          reject(err);
        }
      });
    }
  });
}
