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
    height: 54px;
    width: 54px;
    object-fit: contain;
    border-radius: 6px;
    margin-right: 20px;
  `;

  const container = item.querySelector('.Box') ?? item;

  // @ts-ignore
  container.style = css`
    display: flex;
    align-items: center;
  `;

  container.prepend(img);
});
