import { onElement, setImageSlug, css } from '../utils';

onElement('.js-notifications-group', (notification) => {
  const slug = notification.querySelector('a')!.textContent!.trim();

  const img = document.createElement('img');
  setImageSlug(img, slug);

  // @ts-ignore
  img.style = css`
    height: 26px;
    width: 26px;
    object-fit: contain;
    position: absolute;
    top: -4.5px;
    left: -6.5px;
    border-radius: 6px;
  `;

  const heading = notification.querySelector('h6')!;

  // @ts-ignore
  heading.style = css`
    position: relative;
    padding-left: 30px;
  `;

  heading.prepend(img);
});

onElement('.js-notification-sidebar-repositories', (repos) => {
  for (const repo of repos.querySelectorAll('a')) {
    const slug = repo.lastChild!.textContent!.trim();
    const img = repo.querySelector('img')!;
    setImageSlug(img, slug);
  }
});
