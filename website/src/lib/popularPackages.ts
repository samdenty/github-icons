export async function getPopularPackages() {
  const params = new URLSearchParams([
    ['highlightPreTag', '<ais-highlight-0000000000>'],
    ['highlightPostTag', '</ais-highlight-0000000000>'],
    ['hitsPerPage', '1000'],
    ['page', '0'],
    ['analyticsTags', '["yarnpkg.com"]'],
    [
      'attributesToRetrieve',
      '["deprecated","description","downloadsLast30Days","homepage","humanDownloadsLast30Days","keywords","license","modified","name","owner","repository","types","version"]',
    ],
    ['attributesToHighlight', '["name","description","keywords"]'],
    ['maxValuesPerFacet', '5'],
    ['facets', '["keywords","keywords","owner.name"]'],
    ['tagFilters', ''],
  ]);

  const data = await fetch(
    'https://ofcncog2cu-dsn.algolia.net/1/indexes/*/queries?x-algolia-agent=Algolia%20for%20JavaScript%20(4.2.0)%3B%20Browser%20(lite)%3B%20JS%20Helper%20(3.1.1)%3B%20react%20(16.13.1)%3B%20react-instantsearch%20(6.6.0)&x-algolia-api-key=f54e21fa3a2a0160595bb058179bfb1e&x-algolia-application-id=OFCNCOG2CU',
    {
      method: 'POST',
      body: JSON.stringify({
        requests: [
          {
            indexName: 'npm-search',
            params: params.toString(),
          },
        ],
      }),
    }
  ).then((res) => res.json());

  const hits: { name: string; downloadsLast30Days: number }[] =
    data.results[0].hits;

  hits.sort((a, b) => b.downloadsLast30Days - a.downloadsLast30Days);

  return hits.map((hit) => hit.name);
}
