const fetch = globalThis.fetch;

globalThis.fetch = async (resource, ...args) => {
  let url = resource;
  if (resource instanceof Request) {
    url = resource.url;
  }

  try {
    return await fetch(resource, ...args);
  } catch (e) {
    e.message = `${e.message} (${url})`;
    throw e;
  }
};
