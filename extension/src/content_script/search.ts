import { onElement, slugImage, css } from '../utils';

onElement('.repo-list', (repos) => {
  for (const repo of repos.querySelectorAll('.repo-list-item')) {
    const slug = repo.querySelector('a')!.textContent!.trim();

    const img = document.createElement('img');
    slugImage(slug, img);

    // @ts-ignore
    img.style = css`
      height: 50px;
      width: 50px;
      object-fit: contain;
      margin-right: 20px;
    `;

    repo.firstElementChild!.replaceWith(img);
  }
});
