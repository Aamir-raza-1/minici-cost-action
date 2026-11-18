export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (url.pathname === "/health" || url.pathname === "/api/health") {
      return new Response(
        JSON.stringify({
          status: "ok",
          service: "minici-cost-action",
          version: "0.7.0",
          engine: "Pure Rust std-only (<1ms)",
          providers: 7,
          skus: 160,
          models: 20,
          author: "Aamir Raza <raza.aamir01@gmail.com>"
        }),
        { headers: { "content-type": "application/json; charset=utf-8" } }
      );
    }
    return env.ASSETS.fetch(request);
  }
};
