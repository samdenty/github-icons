import { onElement, slugImage, css } from '../utils';

onElement('.js-pinned-items-reorder-container li', (item) => {
  const segments = new URL(
    item.querySelector('a')!.href,
    location.href
  ).pathname.split('/');
  segments.shift();
  const slug = segments.join('/');

  const img = document.createElement('img');
  slugImage(slug, img);

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

  // @ts-ignore
  container.firstElementChild!.style = css`
    flex-grow: 1;
  `;

  container.prepend(img);
});
