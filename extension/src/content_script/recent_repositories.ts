import { onElement, setImageSlug } from '../utils';

onElement('.js-repos-container[aria-label="Repositories"]', (repos) => {
  for (const source of repos.querySelectorAll('.source')) {
    const slug = source.querySelector('.markdown-title')!.textContent!.trim();
    const img = source.querySelector('img')!;

    setImageSlug(img, slug);
  }
});
