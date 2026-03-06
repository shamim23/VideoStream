import { c as create_ssr_component, b as subscribe, d as add_attribute, e as escape } from "../../../../chunks/ssr.js";
import { p as page } from "../../../../chunks/stores.js";
const css = {
  code: "body{font-family:-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;margin:0;padding:0;background:#000}main.svelte-w0d5zs.svelte-w0d5zs{max-width:1200px;margin:0 auto;padding:1rem}.video-container.svelte-w0d5zs.svelte-w0d5zs{background:#000;border-radius:8px;overflow:hidden}video.svelte-w0d5zs.svelte-w0d5zs{display:block;max-height:80vh;margin:0 auto}.info.svelte-w0d5zs.svelte-w0d5zs{background:#1a1a1a;color:#fff;padding:1.5rem;border-radius:8px;margin-top:1rem}.info.svelte-w0d5zs h1.svelte-w0d5zs{margin:0 0 0.5rem 0}.info.svelte-w0d5zs p.svelte-w0d5zs{color:#999;margin:0.5rem 0}.info.svelte-w0d5zs code.svelte-w0d5zs{background:#333;padding:0.2rem 0.4rem;border-radius:4px}.info.svelte-w0d5zs a.svelte-w0d5zs{color:#4CAF50;text-decoration:none;display:inline-block;margin-top:1rem}.info.svelte-w0d5zs a.svelte-w0d5zs:hover{text-decoration:underline}",
  map: null
};
const Page = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $page, $$unsubscribe_page;
  $$unsubscribe_page = subscribe(page, (value) => $page = value);
  const videoId = $page.params.id;
  const apiBaseUrl = "http://127.0.0.1:3000";
  const videoUrl = `${apiBaseUrl}/api/watch/${videoId}`;
  $$result.css.add(css);
  $$unsubscribe_page();
  return `<main class="svelte-w0d5zs"><div class="video-container svelte-w0d5zs"><video${add_attribute("src", videoUrl, 0)} controls width="100%" autoplay class="svelte-w0d5zs">Your browser does not support the video tag.</video></div> <div class="info svelte-w0d5zs"><h1 class="svelte-w0d5zs" data-svelte-h="svelte-13seavs">Video Player</h1> <p class="svelte-w0d5zs">Video ID: <code class="svelte-w0d5zs">${escape(videoId)}</code></p> <a href="/" class="svelte-w0d5zs" data-svelte-h="svelte-1x8gvfy">← Upload another video</a></div> </main>`;
});
export {
  Page as default
};
