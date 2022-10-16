import { onElement, setImageSlug, css } from '../utils';

onElement('.js-pinned-items-reorder-container li', (item) => {
  const segments = new URL(
    item.querySelector('a')!.href,
    location.href
  ).pathname.split('/');
  segments.shift();
  const slug = segments.join('/');

  const img = document.createElement('img');
  setImageSlug(img, slug);

  // @ts-ignore
  img.style = css`
    max-height: 54px;
    max-width: 54px;
    border-radius: 6px;
    margin-right: 20px;
  `;

  const container = item.querySelector('.Box') ?? item;

  // @ts-ignore
  container.style = css`
    display: flex;
    align-items: center;
  `;

  // @ts-ignore
  container.firstElementChild!.style = css`
    flex-grow: 1;
  `;

  container.prepend(img);
});
