export function slugImage(
  slug: string,
  img: HTMLImageElement = document.createElement('img')
): Promise<string> {
  const [owner] = slug.split('/');

  img.crossOrigin = 'anonymous';
  img.src = `https://github-icons.com/${slug.toLowerCase()}`;

  let errored = false;

  img.onerror = () => {
    if (errored) {
      return;
    }

    errored = true;
    img.src = `https://github.com/${owner}.png`;
  };

  const canvas = document.createElement('canvas');
  const context = canvas.getContext('2d')!;

  return new Promise((resolve) => {
    img.onload = () => {
      canvas.width = img.width;
      canvas.height = img.height;

      context.drawImage(img, 0, 0);
      const dataURL = canvas.toDataURL();

      resolve(dataURL);
    };
  });
}
