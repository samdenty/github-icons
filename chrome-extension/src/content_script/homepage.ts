import { onElement, slugImage, css } from '../utils';

// Recent repositories
onElement('.js-repos-container[aria-label="Repositories"]', (repos) => {
  for (const source of repos.querySelectorAll('.source')) {
    const slug = source.querySelector('.markdown-title')!.textContent!.trim();

    const img = source.querySelector('img')!;
    slugImage(slug, img);
  }
});

// Recent activity
onElement(
  '.js-repos-container[aria-label="Repositories"] + * [data-repository-hovercards-enabled] > *',
  (recentActivity) => {
    const slugAndNumber = recentActivity.querySelectorAll('a')[1]!.textContent!;
    const slug = slugAndNumber.split('#')[0].trim();

    const img = document.createElement('img');
    slugImage(slug, img);

    // @ts-ignore
    recentActivity.style = css`
      position: relative;
      padding-left: 27px;
      margin-top: 16px !important;
    `;

    // @ts-ignore
    img.style = css`
      height: 16px;
      width: 16px;
      border-radius: 3px;
      margin-bottom: 4px;
    `;

    const link = recentActivity.querySelector('a')!;

    // @ts-ignore
    link.style = css`
      flex-direction: column;
      display: flex;
      position: absolute !important;
      top: 3.5px;
      left: 0;
    `;

    link.prepend(img);

    const content = recentActivity.querySelector('div')!;

    // @ts-ignore
    content.style = css`
      overflow: hidden;
      text-overflow: ellipsis;
    `;

    const [, text] = content.querySelectorAll('a');

    // @ts-ignore
    text.style = css`
      white-space: pre;
    `;

    text.textContent = text.textContent!.trim();
  }
);

// Explore repositories
onElement('aside[aria-label="Explore"]', (explore) => {
  for (const repo of explore.querySelectorAll('.py-2.my-2')) {
    let link = repo.querySelector('a')!;
    const slug = link.textContent!.trim();

    const img = document.createElement('img');
    slugImage(slug, img);

    // @ts-ignore
    img.style = css`
      height: 54px;
      width: 54px;
      object-fit: contain;
      margin-right: 20px;
      border-radius: 6px;
    `;

    // @ts-ignore
    repo.style = css`
      display: flex;
    `;

    const wrapper = document.createElement('div');

    // @ts-ignore
    wrapper.style = css`
      font-size: 0 !important;
    `;

    wrapper.append(...repo.childNodes);

    repo.innerHTML = '';

    repo.append(img);
    repo.append(wrapper);
  }
});

// Feed items
onElement('.js-feed-item-view .body > *', (item) => {
  const slug = [...item.querySelectorAll('a')]
    .find((link) => new URL(link.href).pathname.split('/').length === 3)!
    ?.textContent!.trim();

  if (!slug) {
    return;
  }

  const img = document.createElement('img');
  slugImage(slug, img);

  // @ts-ignore
  item.style = css`
    position: relative;
    padding-right: 86px;
  `;

  // @ts-ignore
  img.style = css`
    height: 54px;
    width: 54px;
    object-fit: contain;
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    right: 16px;
    border-radius: 6px;
  `;

  item.prepend(img);
});
