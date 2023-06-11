import { GetServerSideProps, InferGetServerSidePropsType } from 'next';
import { getPopularPackages } from '../lib/popularPackages';
import { Suspense } from 'react';
import IconsGrid from '../components/IconsGrid/IconsGrid';

export const getServerSideProps: GetServerSideProps<{
  packages: string[];
}> = async () => {
  const packages = await getPopularPackages();
  return { props: { packages } };
};

export default function PopularPackages({
  packages,
}: InferGetServerSidePropsType<typeof getServerSideProps>) {
  return (
    <Suspense fallback="loading">
      <IconsGrid icons={packages.map((slug) => ({ type: 'npm', slug }))} />
    </Suspense>
  );
}
