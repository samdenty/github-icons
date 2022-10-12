export function setImageSlug(img: HTMLImageElement, slug: string) {
  const [owner] = slug.split('/');

  img.src = `https://github-icons.com/${slug}`;

  let errored = false;

  img.onerror = () => {
    if (errored) {
      return;
    }

    errored = true;
    img.src = `https://github.com/${owner}.png`;
  };
}
