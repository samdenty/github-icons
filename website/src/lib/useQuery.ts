import { useRouter } from 'next/router';
import { useLayoutEffect, useRef, useState } from 'react';

export function useQuery() {
  const router = useRouter();
  const [query, setQuery] = useState('');

  useLayoutEffect(() => {
    const { query } = router.query;

    if (typeof query === 'string') {
      setQuery(query);
    }
  }, [router.query]);

  useLayoutEffect(() => {
    if (!router.isReady) {
      return;
    }

    if (query) {
      router.query.query = query;
    } else {
      delete router.query.query;
    }

    router.push(router);
  }, [query]);

  return [query, setQuery] as const;
}
