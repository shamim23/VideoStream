

export const index = 2;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_page.svelte.js')).default;
export const imports = ["_app/immutable/nodes/2.DjYxN0xD.js","_app/immutable/chunks/scheduler.C9_6wDfG.js","_app/immutable/chunks/index.CXRiJiS7.js"];
export const stylesheets = ["_app/immutable/assets/2.B9Eu9g6B.css"];
export const fonts = [];
