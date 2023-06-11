import { GetServerSideProps } from 'next';
import { getPopularPackages } from '../lib/popularPackages';

export const getServerSideProps: GetServerSideProps = async ({ res }) => {
  const popularPackages = await getPopularPackages();

  res.setHeader('Content-Type', 'text/xml');

  res.write(`<?xml version="1.0" encoding="UTF-8"?>
  <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    ${popularPackages
      .map((pkg) => {
        return `
      <url>
          <loc>https://github-icons.com/npm/${pkg}</loc>
      </url>
    `;
      })
      .join('')}
  </urlset>
`);

  res.end();

  return { props: {} };
};

// Default export to prevent next.js errors
export default function SitemapIndex() {}
