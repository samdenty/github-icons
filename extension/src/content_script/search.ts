import { onElement, setImageSlug, css } from '../utils';

onElement('.repo-list', (repos) => {
  for (const repo of repos.querySelectorAll('.repo-list-item')) {
    const slug = repo.querySelector('a')!.textContent!.trim();

    const img = document.createElement('img');
    setImageSlug(img, slug);

    // @ts-ignore
    img.style = css`
      max-height: 50px;
      max-width: 50px;
      margin-right: 20px;
    `;

    repo.firstElementChild!.replaceWith(img);
  }
});
