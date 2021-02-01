addEventListener("fetch", (event) => {
  event.respondWith(handleRequest(event.request));
});

const {
  set_token,
  get_icons,
  get_repo_images,
  get_repo_icons,
  get_repo_icon_url,
} = wasm_bindgen;

async function handleRequest(request) {
  await wasm_bindgen(wasm);

  const url = new URL(request.url);

  let token = url.searchParams.get("token");
  if (token) set_token(token);

  let site_url = url.searchParams.get("url");
  if (site_url) {
    return new Response(await get_icons(site_url), { status: 200 });
  }

  const [user, repo, type] = url.pathname.split("/").slice(1);

  if (user && repo) {
    if (type === "images") {
      return new Response(await get_repo_images(user, repo), {
        status: 200,
      });
    }

    if (type === "icons") {
      return new Response(await get_repo_icons(user, repo), {
        status: 200,
      });
    }

    const icon_url = await get_repo_icon_url(user, repo);
    if (!icon_url) {
      return new Response("No icons available for repo", {
        status: 404,
      });
    }
    return Response.redirect(icon_url, 301);
  }
}
