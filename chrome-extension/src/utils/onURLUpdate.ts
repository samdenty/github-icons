const DARK_MODE_QUERY = '(prefers-color-scheme: dark)';
export function onURLUpdate(callback: (isDarkMode: boolean) => void) {
  function onUpdate() {
    callback(window.matchMedia(DARK_MODE_QUERY).matches);
  }

  window.addEventListener('locationchange', onUpdate);
  window.matchMedia(DARK_MODE_QUERY).addEventListener('change', onUpdate);

  onUpdate();
}
