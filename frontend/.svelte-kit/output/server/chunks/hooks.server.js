async function handle({ event, resolve }) {
  if (event.url.pathname.startsWith("/api/")) {
    const backendUrl = `http://localhost:3000${event.url.pathname}${event.url.search}`;
    console.log("Proxying to backend:", backendUrl);
    try {
      const response = await fetch(backendUrl, {
        method: event.request.method,
        headers: event.request.headers,
        body: event.request.body,
        duplex: "half"
      });
      return response;
    } catch (err) {
      console.error("Proxy error:", err);
      return new Response(JSON.stringify({ error: "Backend unavailable" }), {
        status: 502,
        headers: { "content-type": "application/json" }
      });
    }
  }
  return resolve(event);
}
export {
  handle
};
