export function onElement<Type extends HTMLElement>(
  selector: string,
  callback?: (element: Type) => void
) {
  const elements = new Set();

  return new Promise<Type>((resolve) => {
    function update() {
      const newElements = document.querySelectorAll<Type>(selector);

      for (const newElement of newElements) {
        if (elements.has(newElement)) {
          continue;
        }

        resolve(newElement);
        callback?.(newElement);
        elements.add(newElement);
      }
    }
    update();

    const observer = new MutationObserver(() => {
      update();
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true,
    });
  });
}
