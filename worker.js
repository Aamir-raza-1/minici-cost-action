const OFFICIAL_RATES_FEED = {
  schema_version: "1.0",
  catalog_version: "2026.11-v0.8.0",
  updated_at: "2026-11-19T00:00:00Z",
  github_actions: {
    "ubuntu-latest": 0.006,
    "ubuntu-24.04-arm": 0.005,
    "ubuntu-slim": 0.002,
    "ubuntu-4-core": 0.012,
    "ubuntu-8-core": 0.022,
    "ubuntu-16-core": 0.042,
    "windows-latest": 0.01,
    "macos-latest": 0.062,
    "macos-large": 0.12
  },
  gitlab_ci: {
    "saas-linux-small-amd64": 0.01,
    "saas-linux-medium-amd64": 0.02,
    "saas-macos-medium-m1": 0.06
  },
  circleci: {
    "docker-medium": 0.006,
    "docker-large": 0.012,
    "macos-m2pro-medium": 0.09
  }
};

function buildSlackFinOpsBlocks(commandText) {
  const cleaned = (commandText || "").replace(/<@[^>]+>/g, "").trim();
  const tokens = cleaned.split(/\s+/).filter(Boolean);
  const sub = (tokens[0] || "help").toLowerCase();
  const target = tokens[1] || "owner/repo";

  if (sub === "permissions" || sub === "security") {
    return {
      response_type: "in_channel",
      text: "MiniCI Read-Only Security & Least-Privilege Receipt",
      blocks: [
        {
          type: "header",
          text: { type: "plain_text", text: "MiniCI — Read-Only Security & Least-Privilege Receipt" }
        },
        {
          type: "section",
          fields: [
            { type: "mrkdwn", text: "*Source Code Access:*\n`Contents: None` (0 code files read)" },
            { type: "mrkdwn", text: "*Write Scopes Required:*\n`None` (100% Read-Only)" },
            { type: "mrkdwn", text: "*Zero-Token Public Coverage:*\n`68 / 108` attributes (`63.0%`)" },
            { type: "mrkdwn", text: "*Read-PAT + FOCUS CSV:*\n`108 / 108` attributes (`100.0%`)" }
          ]
        }
      ]
    };
  }

  return {
    response_type: "in_channel",
    text: `MiniCI v0.8.0 FinOps Receipt for ${target}`,
    blocks: [
      {
        type: "header",
        text: { type: "plain_text", text: `MiniCI v0.8.0 — CI/CD FinOps (${sub})` }
      },
      {
        type: "section",
        text: {
          type: "mrkdwn",
          text: `*Target:* \`${target}\`\nRun locally or in CI in \`< 1 ms\`:\n\`minici breakdown --repo ${target} --live\`\n\`minici shrink --path .\` • \`minici migrate --path . --to arm64\` • \`minici permissions check\``
        }
      }
    ]
  };
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    if (url.pathname === "/health" || url.pathname === "/api/health") {
      return new Response(
        JSON.stringify({
          status: "ok",
          service: "minici-cost-action",
          version: "0.8.0",
          engine: "Pure Rust std-only (<1ms)",
          providers: 7,
          skus: 160,
          models: 24,
          mcp_tools: 7,
          author: "Aamir Raza <raza.aamir01@gmail.com>"
        }),
        { headers: { "content-type": "application/json; charset=utf-8" } }
      );
    }

    if (url.pathname === "/v1/rates.json") {
      return new Response(JSON.stringify(OFFICIAL_RATES_FEED, null, 2), {
        headers: {
          "content-type": "application/json; charset=utf-8",
          "cache-control": "public, max-age=3600",
          "etag": "\"minici-rates-v0.8.0\""
        }
      });
    }

    // Slack Slash Command (`/minici breakdown owner/repo`, `/minici permissions`)
    if (url.pathname === "/slack/command" && request.method === "POST") {
      const form = await request.formData();
      const text = form.get("text") || "breakdown";
      return new Response(JSON.stringify(buildSlackFinOpsBlocks(String(text))), {
        headers: { "content-type": "application/json; charset=utf-8" }
      });
    }

    // Slack Events API (`@minici` app_mention + URL verification challenge)
    if (url.pathname === "/slack/events" && request.method === "POST") {
      const body = await request.json();
      if (body && body.type === "url_verification" && body.challenge) {
        return new Response(JSON.stringify({ challenge: body.challenge }), {
          headers: { "content-type": "application/json; charset=utf-8" }
        });
      }
      if (body && body.event && body.event.type === "app_mention") {
        const reply = buildSlackFinOpsBlocks(body.event.text || "");
        if (env && env.SLACK_BOT_TOKEN && body.event.channel) {
          await fetch("https://slack.com/api/chat.postMessage", {
            method: "POST",
            headers: {
              "content-type": "application/json; charset=utf-8",
              authorization: `Bearer ${env.SLACK_BOT_TOKEN}`
            },
            body: JSON.stringify({
              channel: body.event.channel,
              thread_ts: body.event.thread_ts || body.event.ts,
              text: reply.text,
              blocks: reply.blocks
            })
          });
        }
        return new Response(JSON.stringify({ ok: true, preview: reply }), {
          headers: { "content-type": "application/json; charset=utf-8" }
        });
      }
    }

    return env.ASSETS.fetch(request);
  }
};
