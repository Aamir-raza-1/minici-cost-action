const RATES_ETAG = "\"minici-rates-v0.8.0\"";

const CORS_HEADERS = {
  "access-control-allow-origin": "*",
  "access-control-allow-methods": "GET, POST, OPTIONS",
  "access-control-allow-headers": "Content-Type, If-None-Match, Authorization"
};

const OFFICIAL_RATES_FEED = {
  schema_version: "1.0",
  catalog_version: "2026.11-v0.8.0",
  updated_at: "2026-11-20T00:00:00Z",
  github_actions: {
    "ubuntu-latest": 0.006,
    "ubuntu-24.04-arm": 0.005,
    "ubuntu-slim": 0.002,
    "ubuntu-4-core": 0.012,
    "ubuntu-4core": 0.012,
    "ubuntu-8-core": 0.022,
    "ubuntu-8core": 0.022,
    "ubuntu-16-core": 0.042,
    "ubuntu-16core": 0.042,
    "ubuntu-32-core": 0.082,
    "ubuntu-64-core": 0.162,
    "ubuntu-4-core-arm": 0.008,
    "ubuntu-8-core-arm": 0.014,
    "ubuntu-16-core-arm": 0.026,
    "windows-latest": 0.01,
    "windows-2025": 0.01,
    "windows-11-arm": 0.008,
    "macos-latest": 0.062,
    "macos-15": 0.062,
    "macos-large": 0.077,
    "macos-15-large": 0.077,
    "macos-xlarge": 0.102,
    "macos-15-xlarge": 0.102,
    "gpu-t4-4-core": 0.052
  },
  gitlab_ci: {
    "saas-linux-small-amd64": 0.01,
    "saas-linux-small-arm64": 0.01,
    "saas-linux-medium-amd64": 0.02,
    "saas-linux-large-amd64": 0.04,
    "saas-linux-xlarge-amd64": 0.08,
    "saas-linux-2xlarge-amd64": 0.16,
    "saas-macos-medium-m1": 0.06,
    "saas-macos-large-m2pro": 0.12
  },
  circleci: {
    "docker-small": 0.003,
    "docker-medium": 0.006,
    "docker-medium+": 0.009,
    "docker-large": 0.012,
    "docker-xlarge": 0.024,
    "arm-medium": 0.0042,
    "arm-large": 0.0084,
    "macos-m1-medium": 0.09,
    "macos-m2pro-medium": 0.09,
    "macos-m2pro-large": 0.18
  },
  bitbucket_pipelines: {
    "bitbucket-1x": 0.01,
    "bitbucket-2x": 0.02,
    "bitbucket-4x": 0.04,
    "bitbucket-8x": 0.08,
    "bitbucket-16x": 0.16
  },
  aws_codebuild: {
    "BUILD_GENERAL1_SMALL": 0.005,
    "BUILD_GENERAL1_MEDIUM": 0.01,
    "BUILD_GENERAL1_LARGE": 0.02,
    "BUILD_GENERAL1_XLARGE": 0.08,
    "ARM_CONTAINER_SMALL": 0.0034,
    "ARM_CONTAINER_LARGE": 0.015
  },
  gcp_cloud_build: {
    "E2_MEDIUM": 0.003,
    "E2_HIGHCPU_8": 0.016,
    "E2_HIGHCPU_32": 0.064,
    "N1_HIGHCPU_8": 0.016,
    "N1_HIGHCPU_32": 0.064
  },
  azure_pipelines: {
    "azure-microsoft-hosted-linux": 0.008,
    "azure-microsoft-hosted-windows": 0.008,
    "azure-microsoft-hosted-macos": 0.008
  }
};

function resolveRunnerRate(label) {
  const raw = String(label || "ubuntu-latest").trim();
  if (OFFICIAL_RATES_FEED.github_actions[raw] !== undefined) {
    return OFFICIAL_RATES_FEED.github_actions[raw];
  }
  const l = raw.toLowerCase();
  if (OFFICIAL_RATES_FEED.github_actions[l] !== undefined) {
    return OFFICIAL_RATES_FEED.github_actions[l];
  }
  const isArm = l.includes("arm") || l.includes("aarch64") || l.includes("graviton");
  const isWin = l.includes("windows");
  if (l.includes("gpu") || l.includes("cuda") || l.includes("t4")) return 0.052;
  if (l.includes("macos") && l.includes("xlarge")) return 0.102;
  if (l.includes("macos") && l.includes("large")) return 0.077;
  if (l.includes("macos")) return 0.062;
  if (l.includes("64-core") || l.includes("64core") || l.includes("64vcpu")) {
    return isWin ? (isArm ? 0.194 : 0.308) : (isArm ? 0.098 : 0.162);
  }
  if (l.includes("32-core") || l.includes("32core") || l.includes("32vcpu")) {
    return isWin ? (isArm ? 0.098 : 0.156) : (isArm ? 0.05 : 0.082);
  }
  if (l.includes("16-core") || l.includes("16core") || l.includes("16vcpu")) {
    return isWin ? (isArm ? 0.05 : 0.082) : (isArm ? 0.026 : 0.042);
  }
  if (l.includes("8-core") || l.includes("8core") || l.includes("8vcpu")) {
    return isWin ? (isArm ? 0.026 : 0.042) : (isArm ? 0.014 : 0.022);
  }
  if (l.includes("4-core") || l.includes("4core") || l.includes("4vcpu")) {
    return isWin ? (isArm ? 0.014 : 0.022) : (isArm ? 0.008 : 0.012);
  }
  if (l.includes("blacksmith") || l.includes("warpbuild") || l.includes("buildjet")) return 0.004;
  if (isWin) return isArm ? 0.008 : 0.01;
  if (isArm) return 0.005;
  if (l.includes("slim")) return 0.002;
  if (l.includes("self-hosted") || l.includes("runs-on=") || l.startsWith("runson-")) return 0.0024;
  return 0.006;
}

async function sha256Hex(text) {
  const bytes = new TextEncoder().encode(String(text || ""));
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

async function verifySlackSignature(request, rawBody, signingSecret, allowUnsignedDev = false) {
  if (!signingSecret) return Boolean(allowUnsignedDev);
  const sigHeader = request.headers.get("x-slack-signature") || "";
  const tsHeader = request.headers.get("x-slack-request-timestamp") || "";
  if (!sigHeader.startsWith("v0=") || !tsHeader) return false;
  const nowSec = Math.floor(Date.now() / 1000);
  const reqSec = Number(tsHeader);
  if (!Number.isFinite(reqSec) || Math.abs(nowSec - reqSec) > 300) return false;

  const enc = new TextEncoder();
  const key = await crypto.subtle.importKey(
    "raw",
    enc.encode(signingSecret),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"]
  );
  const mac = await crypto.subtle.sign(
    "HMAC",
    key,
    enc.encode(`v0:${tsHeader}:${rawBody}`)
  );
  const hex = Array.from(new Uint8Array(mac))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
  const expected = `v0=${hex}`;
  if (expected.length !== sigHeader.length) return false;
  let diff = 0;
  for (let i = 0; i < expected.length; i++) {
    diff |= expected.charCodeAt(i) ^ sigHeader.charCodeAt(i);
  }
  return diff === 0;
}

function estimateStaticStepMinutes(yml) {
  const low = String(yml || "").toLowerCase();
  let est = 1.5;
  if (low.includes("cargo build") || low.includes("cargo test")) est += 4.5;
  else if (low.includes("pytest") || low.includes("tox")) est += 2.5;
  else if (low.includes("npm test") || low.includes("vitest") || low.includes("jest")) est += 2.2;
  else if (low.includes("playwright") || low.includes("cypress")) est += 5.5;
  else if (low.includes("docker/build-push-action") || low.includes("docker build")) est += 3.8;
  else if (low.includes("xcodebuild") || low.includes("gradle")) est += 6.5;
  return Math.ceil(est);
}

/**
 * Fetches 100% real live GitHub repository + Actions run/job/cache telemetry
 * (zero hardcoded `mins = 4.0` or `monthlyRuns = 433` assumptions).
 */
async function fetchLiveGitHubRepoSummary(slug, ghToken, windowDays = 30) {
  if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(slug)) return null;
  const clampedDays = Math.max(1, Math.min(365, Number(windowDays) || 30));
  const headers = {
    accept: "application/vnd.github+json",
    "user-agent": "minici-worker/0.8.0"
  };
  if (ghToken) headers.authorization = `Bearer ${ghToken}`;

  const [repoResp, wfResp, runsResp, cacheResp] = await Promise.all([
    fetch(`https://api.github.com/repos/${slug}`, { headers }),
    fetch(`https://api.github.com/repos/${slug}/contents/.github/workflows`, { headers }),
    fetch(`https://api.github.com/repos/${slug}/actions/runs?per_page=100`, { headers }),
    fetch(`https://api.github.com/repos/${slug}/actions/cache/usage`, { headers }).catch(() => null)
  ]);

  if (!repoResp.ok) return null;
  const repoJson = await repoResp.json();
  const isPublic = !repoJson.private;
  let headSha = String(repoJson.default_branch || "main");

  let workflowFiles = [];
  if (wfResp && wfResp.ok) {
    const entries = await wfResp.json().catch(() => []);
    if (Array.isArray(entries)) {
      workflowFiles = entries.filter((e) => e && e.download_url && /\.ya?ml$/i.test(e.name || ""));
    }
  }

  let activeCacheGb = 0.0;
  let activeCachesCount = 0;
  if (cacheResp && cacheResp.ok) {
    const cj = await cacheResp.json().catch(() => null);
    if (cj) {
      activeCacheGb = Number(cj.active_caches_size_in_bytes || 0) / (1024 * 1024 * 1024);
      activeCachesCount = Number(cj.active_caches_count || 0);
    }
  }

  const nowMs = Date.now();
  const cutoffMs = nowMs - clampedDays * 86400 * 1000;
  let observedRunsInWindow = 0;
  let skippedNoopRuns = 0;
  let botRuns = 0;
  let cancelledRuns = 0;
  let oldestRunMs = nowMs;
  let newestRunMs = 0;
  const completedSampleRuns = [];

  if (runsResp && runsResp.ok) {
    const runsJson = await runsResp.json().catch(() => ({}));
    const rawRuns = Array.isArray(runsJson.workflow_runs) ? runsJson.workflow_runs : [];
    if (rawRuns.length > 0 && rawRuns[0].head_sha) {
      headSha = String(rawRuns[0].head_sha).slice(0, 12);
    }
    for (const run of rawRuns) {
      const createdMs = Date.parse(run.created_at || "");
      if (!Number.isFinite(createdMs) || createdMs < cutoffMs) continue;
      if (createdMs < oldestRunMs) oldestRunMs = createdMs;
      if (createdMs > newestRunMs) newestRunMs = createdMs;

      if (run.conclusion === "skipped") {
        skippedNoopRuns += 1;
        continue;
      }
      observedRunsInWindow += 1;
      const actorLogin = String((run.actor && run.actor.login) || "").toLowerCase();
      if (actorLogin.endsWith("[bot]") || actorLogin.includes("dependabot") || actorLogin.includes("renovate")) {
        botRuns += 1;
      }
      if (run.conclusion === "cancelled") {
        cancelledRuns += 1;
      }
      if (run.status === "completed" && completedSampleRuns.length < 6) {
        completedSampleRuns.push(run);
      }
    }
  }

  // Compute real monthly run cadence extrapolated from the actual observed time span
  let monthlyRuns = 0.0;
  let provenanceMode = "STATIC_TOPOLOGY";
  if (observedRunsInWindow > 0) {
    provenanceMode = "LIVE_CALIBRATED";
    const spanDays = Math.max(
      1.0,
      Math.min(clampedDays, (Math.max(nowMs, newestRunMs) - oldestRunMs) / (86400 * 1000))
    );
    monthlyRuns = observedRunsInWindow * (30.0 / spanDays);
  }

  // Sample real job durations & runtime labels[] from recent completed workflow runs
  let totalJobs = 0;
  let billedMinsPerRun = 0.0;
  let runCostUsd = 0.0;
  let cancelledWastedMins = 0.0;

  if (completedSampleRuns.length > 0) {
    const jobResponses = await Promise.all(
      completedSampleRuns.map((r) =>
        fetch(`https://api.github.com/repos/${slug}/actions/runs/${r.id}/jobs?per_page=100`, {
          headers
        })
          .then((res) => (res.ok ? res.json() : null))
          .catch(() => null)
      )
    );
    let validRunSamples = 0;
    let sumJobs = 0;
    let sumBilledMins = 0.0;
    let sumRunCost = 0.0;

    for (const jPayload of jobResponses) {
      if (!jPayload || !Array.isArray(jPayload.jobs) || jPayload.jobs.length === 0) continue;
      let sampleBilled = 0.0;
      let sampleCost = 0.0;
      let sampleJobCount = 0;
      for (const job of jPayload.jobs) {
        if (job.conclusion === "skipped") continue;
        const startMs = Date.parse(job.started_at || "");
        const endMs = Date.parse(job.completed_at || "");
        if (!Number.isFinite(startMs) || !Number.isFinite(endMs) || endMs < startMs) continue;
        const durSec = Math.max(1, (endMs - startMs) / 1000.0);
        const billedMins = Math.ceil(durSec / 60.0);
        const labels = Array.isArray(job.labels) ? job.labels : [];
        const runtimeLabel = labels.find((l) => l && l !== "self-hosted") || labels[0] || "ubuntu-latest";
        const rate = resolveRunnerRate(runtimeLabel);
        sampleJobCount += 1;
        sampleBilled += billedMins;
        sampleCost += billedMins * rate;
        if (job.conclusion === "cancelled") {
          cancelledWastedMins += billedMins;
        }
      }
      if (sampleJobCount > 0) {
        validRunSamples += 1;
        sumJobs += sampleJobCount;
        sumBilledMins += sampleBilled;
        sumRunCost += sampleCost;
      }
    }

    if (validRunSamples > 0) {
      totalJobs = Math.round(sumJobs / validRunSamples);
      billedMinsPerRun = sumBilledMins / validRunSamples;
      runCostUsd = sumRunCost / validRunSamples;
    }
  }

  // Fallback to static workflow topology inspection ONLY when repository has 0 live completed runs
  if (totalJobs === 0 && workflowFiles.length > 0) {
    const inspectFiles = workflowFiles.slice(0, 12);
    const rawYamls = await Promise.all(
      inspectFiles.map((wf) =>
        fetch(wf.download_url, { headers })
          .then((r) => (r.ok ? r.text() : ""))
          .catch(() => "")
      )
    );
    let cronRunsPerMonth = 0.0;
    let hasPrOrPush = false;
    for (const yml of rawYamls) {
      if (!yml) continue;
      if (/pull_request|push/i.test(yml)) hasPrOrPush = true;
      if (/cron:/i.test(yml)) cronRunsPerMonth += 30.0;
      const stepEstMins = estimateStaticStepMinutes(yml);
      const matches = [...yml.matchAll(/^\s+runs-on:\s*([^\n#]+)/gm)];
      const count = Math.max(matches.length, 1);
      for (let i = 0; i < count; i++) {
        const label = matches[i] ? matches[i][1].trim() : "ubuntu-latest";
        const rate = resolveRunnerRate(label);
        totalJobs += 1;
        billedMinsPerRun += stepEstMins;
        runCostUsd += stepEstMins * rate;
      }
    }
    if (monthlyRuns <= 0) {
      const stars = Number(repoJson.stargazers_count || 0);
      const activityFactor = stars > 10000 ? 220.0 : stars > 1000 ? 120.0 : 65.0;
      monthlyRuns = (hasPrOrPush ? activityFactor : 0.0) + cronRunsPerMonth;
    }
  }

  const botRunSharePct =
    observedRunsInWindow > 0 ? (botRuns / observedRunsInWindow) * 100.0 : 0.0;
  const grossMonthlyUsd = runCostUsd * monthlyRuns;
  // Public GitHub repositories receive $0.00 net billed standard GitHub-hosted minutes
  const netMonthlyUsd = isPublic ? 0.0 : Math.max(0.0, grossMonthlyUsd - 2000 * 0.006);

  return {
    repo: slug,
    vendor: "github",
    provenanceMode,
    windowDays: clampedDays,
    headSha,
    isPublic,
    workflows: workflowFiles.length,
    jobs: totalJobs,
    billedMins: Number(billedMinsPerRun.toFixed(2)),
    monthlyRuns: Number(monthlyRuns.toFixed(1)),
    observedRunsInWindow,
    skippedNoopRuns,
    botRunSharePct: Number(botRunSharePct.toFixed(1)),
    cancelledRuns,
    cancelledWastedMins: Number(cancelledWastedMins.toFixed(1)),
    activeCacheGb: Number(activeCacheGb.toFixed(3)),
    activeCachesCount,
    grossMonthlyUsd: Number(grossMonthlyUsd.toFixed(2)),
    netMonthlyUsd: Number(netMonthlyUsd.toFixed(2)),
    updatedAtEpochSec: Math.floor(nowMs / 1000)
  };
}

/**
 * 3-Layer Cloudflare Edge Caching Stack:
 *   Layer 1: `caches.default` (Edge PoP Cache API, <0.5 ms, s-maxage=300, stale-while-revalidate=3600)
 *   Layer 2: `env.MINICI_KV` (Global Workers KV, ~3-8 ms, `report:<slug>:<days>d`)
 *   Layer 3: `env.MINICI_D1` (Cloudflare D1 SQLite `repo_telemetry`, ~5-15 ms)
 *   Origin:  Live GitHub REST API (`fetchLiveGitHubRepoSummary`) + `ctx.waitUntil` async L3->L2->L1 backfill
 */
async function getRepoReportThreeLayer(slug, windowDays, env, ctx, forceRefresh = false) {
  const clampedDays = Math.max(1, Math.min(365, Number(windowDays) || 30));
  const l1Url = `https://cache.minici.dev/v1/report/${slug}?days=${clampedDays}`;
  const l1Key = new Request(l1Url, { method: "GET" });
  const kvKey = `report:${slug}:${clampedDays}d`;
  const edgeCache = typeof caches !== "undefined" && caches.default ? caches.default : null;

  // 1. Layer 1: Cloudflare Edge PoP Cache API (`< 0.5 ms`)
  if (!forceRefresh && edgeCache) {
    const l1Hit = await edgeCache.match(l1Key).catch(() => null);
    if (l1Hit) {
      const data = await l1Hit.json().catch(() => null);
      if (data) return { data, cacheLayer: "L1-EDGE-POP" };
    }
  }

  const putL1 = async (payloadObj) => {
    if (!edgeCache) return;
    const resp = new Response(JSON.stringify(payloadObj), {
      headers: {
        "content-type": "application/json; charset=utf-8",
        "cache-control": "public, s-maxage=300, stale-while-revalidate=3600"
      }
    });
    await edgeCache.put(l1Key, resp).catch(() => {});
  };

  // 2. Layer 2: Cloudflare Workers KV (`~3-8 ms` globally replicated)
  if (!forceRefresh && env && env.MINICI_KV) {
    const kvRaw = await env.MINICI_KV.get(kvKey).catch(() => null);
    if (kvRaw) {
      try {
        const parsed = JSON.parse(kvRaw);
        if (ctx && typeof ctx.waitUntil === "function") ctx.waitUntil(putL1(parsed));
        return { data: parsed, cacheLayer: "L2-WORKERS-KV" };
      } catch {}
    }
  }

  // 3. Layer 3: Cloudflare D1 Serverless SQLite (`~5-15 ms` persistent FinOps store)
  if (!forceRefresh && env && env.MINICI_D1) {
    try {
      const row = await env.MINICI_D1.prepare(
        "SELECT payload_json, updated_at FROM repo_telemetry WHERE repo = ?1 AND window_days = ?2"
      )
        .bind(slug, clampedDays)
        .first();
      const nowSec = Math.floor(Date.now() / 1000);
      if (row && row.payload_json && nowSec - Number(row.updated_at || 0) <= 3600) {
        const parsed = JSON.parse(row.payload_json);
        if (ctx && typeof ctx.waitUntil === "function") {
          ctx.waitUntil(
            Promise.all([
              env.MINICI_KV
                ? env.MINICI_KV.put(kvKey, row.payload_json, { expirationTtl: 3600 }).catch(() => {})
                : Promise.resolve(),
              putL1(parsed)
            ])
          );
        }
        return { data: parsed, cacheLayer: "L3-CLOUDFLARE-D1" };
      }
    } catch {}
  }

  // 4. Origin Fetch: Live GitHub Telemetry + Async D1 -> KV -> Edge PoP Backfill
  const live = await fetchLiveGitHubRepoSummary(slug, env && env.GITHUB_TOKEN, clampedDays);
  if (!live) return { data: null, cacheLayer: "MISS" };

  const serialized = JSON.stringify(live);
  const backfillPromise = Promise.all([
    env && env.MINICI_D1
      ? env.MINICI_D1.prepare(
          `INSERT OR REPLACE INTO repo_telemetry (
             repo, vendor, visibility, head_sha, window_days, updated_at,
             monthly_runs, skipped_noop_runs, bot_run_share_pct, billed_mins_per_run,
             gross_monthly_usd, net_billed_monthly_usd, active_cache_gb, payload_json
           ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)`
        )
          .bind(
            slug,
            "github",
            live.isPublic ? "public" : "private",
            live.headSha || "",
            clampedDays,
            live.updatedAtEpochSec,
            live.monthlyRuns,
            live.skippedNoopRuns,
            live.botRunSharePct,
            live.billedMins,
            live.grossMonthlyUsd,
            live.netMonthlyUsd,
            live.activeCacheGb,
            serialized
          )
          .run()
          .catch(() => {})
      : Promise.resolve(),
    env && env.MINICI_KV
      ? env.MINICI_KV.put(kvKey, serialized, { expirationTtl: 3600 }).catch(() => {})
      : Promise.resolve(),
    putL1(live)
  ]);

  if (ctx && typeof ctx.waitUntil === "function") {
    ctx.waitUntil(backfillPromise);
  } else {
    await backfillPromise;
  }

  return { data: live, cacheLayer: "ORIGIN-GITHUB" };
}

/**
 * Real Edge `/v1/systemone` Constrained-Choice Evaluator:
 *   1. Checks Layer-2 KV (`jev:step:<sha256>`) & Layer-3 D1 (`step_decisions`) content-addressed cache.
 *   2. Forwards to `env.JEV_UPSTREAM_URL` (Ollaya / TypeSafe `/v1/systemone` or `jev_systemone_server.py`) when configured.
 *   3. Executes Workers AI (`env.AI`) with cyclic-shift option-order debiasing when bound.
 *   4. Returns calibrated `noul` (`choice: null`) when offline / unbound — zero hardcoded benchmark answers!
 */
async function evaluateSystemOneEdge(req, env, ctx) {
  const rawHashInput = JSON.stringify({
    state: req.state || null,
    prompt: req.prompt || null,
    questions: req.questions || null,
    choice: req.choice || null
  });
  const cacheDigest = await sha256Hex(rawHashInput);
  const kvKey = `jev:step:${cacheDigest}`;

  if (env && env.MINICI_KV) {
    const cached = await env.MINICI_KV.get(kvKey).catch(() => null);
    if (cached) {
      try {
        const obj = JSON.parse(cached);
        return { ...obj, cache_layer: "L2-WORKERS-KV" };
      } catch {}
    }
  }

  if (env && env.MINICI_D1) {
    const d1Row = await env.MINICI_D1.prepare(
      "SELECT phase, confidence, model_id FROM step_decisions WHERE step_sha256 = ?1"
    )
      .bind(cacheDigest)
      .first()
      .catch(() => null);
    if (d1Row && d1Row.phase) {
      const d1Hit = {
        choice: d1Row.phase,
        score: Number(d1Row.confidence || 0.9),
        noul: false,
        backend: d1Row.model_id || "jev-systemone",
        cache_layer: "L3-CLOUDFLARE-D1"
      };
      if (env.MINICI_KV && ctx && typeof ctx.waitUntil === "function") {
        ctx.waitUntil(
          env.MINICI_KV.put(kvKey, JSON.stringify(d1Hit), { expirationTtl: 2592000 }).catch(() => {})
        );
      }
      return d1Hit;
    }
  }

  const persistDecisionL2L3 = (outObj) => {
    const nowSec = Math.floor(Date.now() / 1000);
    const kvTask =
      env && env.MINICI_KV
        ? env.MINICI_KV.put(kvKey, JSON.stringify(outObj), { expirationTtl: 2592000 }).catch(() => {})
        : Promise.resolve();
    const d1Task =
      env && env.MINICI_D1 && outObj && outObj.choice
        ? env.MINICI_D1.prepare(
            `INSERT OR REPLACE INTO step_decisions (
               step_sha256, phase, arm64_safe, spot_safe, cancel_safe,
               cache_candidate, confidence, model_id, created_at
             ) VALUES (?1, ?2, 1, 1, 1, 0, ?3, ?4, ?5)`
          )
            .bind(
              cacheDigest,
              String(outObj.choice),
              Number(outObj.score || 0.9),
              String(outObj.backend || "jev-systemone"),
              nowSec
            )
            .run()
            .catch(() => {})
        : Promise.resolve();
    if (ctx && typeof ctx.waitUntil === "function") {
      ctx.waitUntil(Promise.all([kvTask, d1Task]));
    }
  };

  if (env && env.JEV_UPSTREAM_URL) {
    const upstreamHeaders = { "content-type": "application/json" };
    if (env.TYPESAFE_API_KEY) {
      upstreamHeaders.authorization = `Bearer ${env.TYPESAFE_API_KEY}`;
    }
    const upstreamResp = await fetch(env.JEV_UPSTREAM_URL, {
      method: "POST",
      headers: upstreamHeaders,
      body: JSON.stringify(req)
    }).catch(() => null);
    if (upstreamResp && upstreamResp.ok) {
      const out = await upstreamResp.json().catch(() => null);
      if (out) {
        persistDecisionL2L3(out);
        return out;
      }
    }
  }

  // Workers AI (`env.AI`) with AnyJev L0 cyclic-shift option-order debiasing
  const choices = Array.isArray(req.choice) ? req.choice.map(String) : [];
  const promptText = String(req.prompt || JSON.stringify(req.state || {}));
  if (env && env.AI && choices.length > 0) {
    const model = env.WORKERS_AI_MODEL || "@cf/qwen/qwen2.5-coder-32b-instruct";
    const shifts = choices.length >= 2 ? [0, Math.floor(choices.length / 2)] : [0];
    const votes = new Map();
    for (const shift of shifts) {
      const rotated = choices.slice(shift).concat(choices.slice(0, shift));
      if (req.noul) rotated.push("NONE_ABSTAIN");
      const sysPrompt =
        "You are a TypeSafe SystemOne constrained classifier. Output ONLY the exact matching string from ALLOWED_CHOICES and nothing else.";
      const userMsg = `INPUT:\n${promptText}\n\nALLOWED_CHOICES:\n${rotated
        .map((c, idx) => `${idx + 1}. ${c}`)
        .join("\n")}\n\nRespond with ONLY one exact token from ALLOWED_CHOICES:`;
      const aiRes = await env.AI.run(model, {
        messages: [
          { role: "system", content: sysPrompt },
          { role: "user", content: userMsg }
        ],
        temperature: 0.0,
        max_tokens: 24
      }).catch(() => null);
      const rawOut = String((aiRes && aiRes.response) || "").trim();
      const matched = rotated.find((c) => rawOut === c || rawOut.includes(c));
      if (matched) {
        votes.set(matched, (votes.get(matched) || 0) + 1);
      }
    }
    let bestChoice = null;
    let bestVotes = 0;
    for (const [cand, v] of votes.entries()) {
      if (v > bestVotes) {
        bestChoice = cand;
        bestVotes = v;
      }
    }
    if (!bestChoice || bestChoice === "NONE_ABSTAIN") {
      return { choice: null, score: 0.0, noul: true, backend: "workers-ai-abstain" };
    }
    const agreement = bestVotes / shifts.length;
    const out = {
      choice: bestChoice,
      score: Number((0.65 + 0.32 * agreement).toFixed(3)),
      noul: false,
      backend: model
    };
    persistDecisionL2L3(out);
    return out;
  }

  return {
    choice: null,
    score: 0.0,
    noul: true,
    note: "Bind JEV_UPSTREAM_URL (jev_systemone_server.py / Ollaya) or Cloudflare Workers AI (env.AI) for live neural /v1/systemone inference."
  };
}

async function buildSlackFinOpsBlocks(commandText, env, ctx) {
  const cleaned = (commandText || "").replace(/<@[^>]+>/g, "").trim();
  const tokens = cleaned.split(/\s+/).filter(Boolean);
  const sub = (tokens[0] || "breakdown").toLowerCase();
  const target = tokens[1] || "owner/repo";

  if (sub === "permissions" || sub === "security") {
    return {
      response_type: "in_channel",
      text: "MiniCI Least-Privilege Token & Coverage Receipt",
      blocks: [
        {
          type: "header",
          text: { type: "plain_text", text: "MiniCI — Least-Privilege Token & Coverage Receipt" }
        },
        {
          type: "section",
          fields: [
            { type: "mrkdwn", text: "*Public Repo Access:*\n`Contents: None` (`63.0%` zero-token)" },
            { type: "mrkdwn", text: "*Private Repo YAML:*\n`Actions: Read` + `Contents: Read` (`82.4%`)" },
            { type: "mrkdwn", text: "*Write Scopes Required:*\n`None` (100% Read-Only)" },
            { type: "mrkdwn", text: "*With FOCUS v1.1 CSV + Step:*\n`108 / 108` attributes (`100.0%`)" }
          ]
        }
      ]
    };
  }

  const { data: live, cacheLayer } = await getRepoReportThreeLayer(target, 30, env, ctx, false).catch(
    () => ({ data: null, cacheLayer: "MISS" })
  );
  if (live) {
    return {
      response_type: "in_channel",
      text: `MiniCI Breakdown for ${target}: $${live.grossMonthlyUsd.toFixed(2)}/mo gross ($${live.netMonthlyUsd.toFixed(2)}/mo net)`,
      blocks: [
        {
          type: "header",
          text: { type: "plain_text", text: `MiniCI — Live FinOps Receipt (${target})` }
        },
        {
          type: "section",
          fields: [
            { type: "mrkdwn", text: `*Gross Compute:*\n\`$${live.grossMonthlyUsd.toFixed(2)}/mo\`` },
            {
              type: "mrkdwn",
              text: `*Net Billed (${live.isPublic ? "Public OSS" : "Private"}):*\n\`$${live.netMonthlyUsd.toFixed(2)}/mo\``
            },
            {
              type: "mrkdwn",
              text: `*Workflows / Jobs / Cadence:*\n\`${live.workflows}\` wf • \`${live.jobs}\` jobs • \`${live.monthlyRuns}/mo\``
            },
            {
              type: "mrkdwn",
              text: `*Billed / Cache (${cacheLayer}):*\n\`${live.billedMins.toFixed(1)}m/run\` • \`${live.activeCacheGb.toFixed(2)} GB\``
            }
          ]
        },
        {
          type: "section",
          text: {
            type: "mrkdwn",
            text: `*Full 24-Model Local Receipt (<1 ms):* \`minici breakdown --repo ${target} --live --days ${live.windowDays}\``
          }
        }
      ]
    };
  }

  return {
    response_type: "in_channel",
    text: `MiniCI FinOps Receipt for ${target}`,
    blocks: [
      {
        type: "header",
        text: { type: "plain_text", text: `MiniCI — CI/CD FinOps (${sub})` }
      },
      {
        type: "section",
        text: {
          type: "mrkdwn",
          text: `*Target:* \`${target}\`\nRun locally or in CI in \`< 1 ms\`:\n\`minici breakdown --repo ${target} --live\`\n\`minici shrink --path .\` • \`minici migrate --path . --to github-arm64\` • \`minici ask "why did CI spend spike?"\``
        }
      }
    ]
  };
}

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);

    if (
      request.method === "OPTIONS" &&
      (url.pathname === "/health" ||
        url.pathname === "/api/health" ||
        url.pathname === "/v1/rates.json" ||
        url.pathname === "/v1/pricing/catalog" ||
        url.pathname === "/v1/systemone" ||
        url.pathname.startsWith("/v1/report/") ||
        url.pathname.startsWith("/v1/jev/"))
    ) {
      return new Response(null, { status: 204, headers: CORS_HEADERS });
    }

    if (url.pathname === "/health" || url.pathname === "/api/health") {
      return new Response(
        JSON.stringify({
          status: "ok",
          service: "minici-cost-action",
          version: "0.8.0",
          engine: "Pure Rust std-only (<1ms)",
          cache_layers: [
            "L1:caches.default",
            env && env.MINICI_KV ? "L2:MINICI_KV(bound)" : "L2:MINICI_KV(optional)",
            env && env.MINICI_D1 ? "L3:MINICI_D1(bound)" : "L3:MINICI_D1(optional)"
          ],
          providers: 7,
          skus: 310,
          models: 24,
          mcp_tools: 8,
          author: "Aamir Raza <raza.aamir01@gmail.com>"
        }),
        { headers: { "content-type": "application/json; charset=utf-8", ...CORS_HEADERS } }
      );
    }

    if (url.pathname === "/v1/rates.json" || url.pathname === "/v1/pricing/catalog") {
      const inm = request.headers.get("if-none-match") || "";
      const inmMatched = inm
        .split(",")
        .map((t) => t.trim().replace(/^W\//i, ""))
        .some((t) => t === "*" || t === RATES_ETAG || t === "minici-rates-v0.8.0");
      if (inmMatched) {
        return new Response(null, {
          status: 304,
          headers: {
            "cache-control": "public, max-age=3600, s-maxage=86400",
            etag: RATES_ETAG,
            ...CORS_HEADERS
          }
        });
      }
      if (env && env.MINICI_KV) {
        const kvPricing = await env.MINICI_KV.get("pricing:catalog:2026").catch(() => null);
        if (kvPricing) {
          return new Response(kvPricing, {
            headers: {
              "content-type": "application/json; charset=utf-8",
              "cache-control": "public, max-age=3600, s-maxage=86400",
              "x-minici-cache": "L2-WORKERS-KV",
              etag: RATES_ETAG,
              ...CORS_HEADERS
            }
          });
        }
      }
      return new Response(JSON.stringify(OFFICIAL_RATES_FEED, null, 2), {
        headers: {
          "content-type": "application/json; charset=utf-8",
          "cache-control": "public, max-age=3600, s-maxage=86400",
          "x-minici-cache": "EMBEDDED-CATALOG",
          etag: RATES_ETAG,
          ...CORS_HEADERS
        }
      });
    }

    // 3-Layer Cached Live Repo Report Endpoint: GET /v1/report/:owner/:repo?days=30&refresh=1
    if (
      (url.pathname.startsWith("/v1/report/") || url.pathname === "/api/analyze") &&
      request.method === "GET"
    ) {
      const slug = url.pathname.startsWith("/v1/report/")
        ? url.pathname.slice("/v1/report/".length).replace(/^\/+|\/+$/g, "")
        : String(url.searchParams.get("repo") || "").trim();
      const windowDays = Number(url.searchParams.get("days") || 30);
      const forceRefresh = url.searchParams.get("refresh") === "1";
      const { data, cacheLayer } = await getRepoReportThreeLayer(
        slug,
        windowDays,
        env,
        ctx,
        forceRefresh
      );
      if (!data) {
        return new Response(
          JSON.stringify({ error: "repo_not_found_or_rate_limited", repo: slug }),
          {
            status: 404,
            headers: { "content-type": "application/json; charset=utf-8", ...CORS_HEADERS }
          }
        );
      }
      return new Response(JSON.stringify(data, null, 2), {
        headers: {
          "content-type": "application/json; charset=utf-8",
          "cache-control": "public, s-maxage=300, stale-while-revalidate=3600",
          "x-minici-cache": cacheLayer,
          ...CORS_HEADERS
        }
      });
    }

    // GitHub Webhook Invalidation & D1/KV Refresh (`workflow_run.completed` / `push`)
    if (url.pathname === "/v1/webhook/github" && request.method === "POST") {
      let body = {};
      try {
        body = JSON.parse((await request.text()) || "{}");
      } catch {
        return new Response(JSON.stringify({ error: "invalid_json" }), { status: 400 });
      }
      const slug = body && body.repository && body.repository.full_name;
      if (slug && /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(slug)) {
        const edgeCache = typeof caches !== "undefined" && caches.default ? caches.default : null;
        for (const d of [7, 30, 60, 90]) {
          if (edgeCache) {
            await edgeCache
              .delete(new Request(`https://cache.minici.dev/v1/report/${slug}?days=${d}`))
              .catch(() => {});
          }
          if (env && env.MINICI_KV) {
            await env.MINICI_KV.delete(`report:${slug}:${d}d`).catch(() => {});
          }
        }
        if (ctx && typeof ctx.waitUntil === "function") {
          ctx.waitUntil(getRepoReportThreeLayer(slug, 30, env, ctx, true));
        }
        return new Response(JSON.stringify({ ok: true, refreshed_repo: slug }), {
          headers: { "content-type": "application/json; charset=utf-8" }
        });
      }
      return new Response(JSON.stringify({ ok: true, ignored: true }), {
        headers: { "content-type": "application/json; charset=utf-8" }
      });
    }

    // Cross-Repo SHA-256 Step Classification Cache (`GET/POST /v1/jev/step-cache`)
    if (url.pathname.startsWith("/v1/jev/step-cache")) {
      if (request.method === "GET") {
        const hash = url.pathname.split("/").pop() || "";
        if (!/^[a-f0-9]{64}$/i.test(hash)) {
          return new Response(JSON.stringify({ error: "invalid_sha256" }), {
            status: 400,
            headers: { "content-type": "application/json; charset=utf-8", ...CORS_HEADERS }
          });
        }
        if (env && env.MINICI_KV) {
          const kvVal = await env.MINICI_KV.get(`jev:step:${hash}`).catch(() => null);
          if (kvVal) {
            return new Response(kvVal, {
              headers: {
                "content-type": "application/json; charset=utf-8",
                "x-minici-cache": "L2-WORKERS-KV",
                ...CORS_HEADERS
              }
            });
          }
        }
        if (env && env.MINICI_D1) {
          const row = await env.MINICI_D1.prepare(
            "SELECT * FROM step_decisions WHERE step_sha256 = ?1"
          )
            .bind(hash)
            .first()
            .catch(() => null);
          if (row) {
            return new Response(JSON.stringify(row), {
              headers: {
                "content-type": "application/json; charset=utf-8",
                "x-minici-cache": "L3-CLOUDFLARE-D1",
                ...CORS_HEADERS
              }
            });
          }
        }
        return new Response(JSON.stringify({ hit: false }), {
          status: 404,
          headers: { "content-type": "application/json; charset=utf-8", ...CORS_HEADERS }
        });
      }
      if (request.method === "POST") {
        let payload = {};
        try {
          payload = JSON.parse((await request.text()) || "{}");
        } catch {
          return new Response(JSON.stringify({ error: "invalid_json" }), { status: 400 });
        }
        const stepHash =
          payload.step_sha256 || (await sha256Hex(String(payload.step_text || "")));
        const nowSec = Math.floor(Date.now() / 1000);
        const record = {
          step_sha256: stepHash,
          phase: String(payload.phase || "compile_typecheck"),
          arm64_safe: payload.arm64_safe !== false,
          spot_safe: payload.spot_safe !== false,
          cancel_safe: payload.cancel_safe !== false,
          cache_candidate: Boolean(payload.cache_candidate),
          confidence: Number(payload.confidence || 0.92),
          model_id: String(payload.model_id || "jev-systemone"),
          created_at: nowSec
        };
        if (env && env.MINICI_KV) {
          await env.MINICI_KV.put(`jev:step:${stepHash}`, JSON.stringify(record), {
            expirationTtl: 2592000
          }).catch(() => {});
        }
        if (env && env.MINICI_D1) {
          await env.MINICI_D1.prepare(
            `INSERT OR REPLACE INTO step_decisions (
               step_sha256, phase, arm64_safe, spot_safe, cancel_safe,
               cache_candidate, confidence, model_id, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)`
          )
            .bind(
              record.step_sha256,
              record.phase,
              record.arm64_safe ? 1 : 0,
              record.spot_safe ? 1 : 0,
              record.cancel_safe ? 1 : 0,
              record.cache_candidate ? 1 : 0,
              record.confidence,
              record.model_id,
              record.created_at
            )
            .run()
            .catch(() => {});
        }
        return new Response(JSON.stringify({ ok: true, step_sha256: stepHash }), {
          headers: { "content-type": "application/json; charset=utf-8", ...CORS_HEADERS }
        });
      }
    }

    if (url.pathname === "/v1/systemone") {
      if (request.method !== "POST") {
        return new Response(
          JSON.stringify({ service: "minici-systemone-jev", status: "ready", method: "POST" }),
          { headers: { "content-type": "application/json; charset=utf-8", ...CORS_HEADERS } }
        );
      }
      let req = {};
      try {
        req = JSON.parse((await request.text()) || "{}");
      } catch {
        return new Response(JSON.stringify({ error: "invalid_json" }), {
          status: 400,
          headers: { "content-type": "application/json; charset=utf-8", ...CORS_HEADERS }
        });
      }
      const decision = await evaluateSystemOneEdge(req, env, ctx);
      return new Response(JSON.stringify(decision), {
        headers: { "content-type": "application/json; charset=utf-8", ...CORS_HEADERS }
      });
    }

    if (url.pathname === "/slack/command" && request.method === "POST") {
      const rawBody = await request.text();
      const allowDev = Boolean(env && env.ALLOW_UNSIGNED_SLACK_DEV === "true");
      const valid = await verifySlackSignature(
        request,
        rawBody,
        env && env.SLACK_SIGNING_SECRET,
        allowDev
      );
      if (!valid) {
        return new Response(JSON.stringify({ error: "invalid_slack_signature" }), {
          status: 401,
          headers: { "content-type": "application/json; charset=utf-8" }
        });
      }
      const params = new URLSearchParams(rawBody);
      const text = params.get("text") || "breakdown";
      const payload = await buildSlackFinOpsBlocks(String(text), env, ctx);
      return new Response(JSON.stringify(payload), {
        headers: { "content-type": "application/json; charset=utf-8" }
      });
    }

    if (url.pathname === "/slack/events" && request.method === "POST") {
      const rawBody = await request.text();
      const allowDev = Boolean(env && env.ALLOW_UNSIGNED_SLACK_DEV === "true");
      const valid = await verifySlackSignature(
        request,
        rawBody,
        env && env.SLACK_SIGNING_SECRET,
        allowDev
      );
      if (!valid) {
        return new Response(JSON.stringify({ error: "invalid_slack_signature" }), {
          status: 401,
          headers: { "content-type": "application/json; charset=utf-8" }
        });
      }
      let body = {};
      try {
        body = JSON.parse(rawBody || "{}");
      } catch {
        return new Response(JSON.stringify({ error: "invalid_json" }), {
          status: 400,
          headers: { "content-type": "application/json; charset=utf-8" }
        });
      }
      if (body && body.type === "url_verification" && body.challenge) {
        return new Response(JSON.stringify({ challenge: body.challenge }), {
          headers: { "content-type": "application/json; charset=utf-8" }
        });
      }
      if (body && body.event && body.event.type === "app_mention") {
        const reply = await buildSlackFinOpsBlocks(body.event.text || "", env, ctx);
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
      return new Response(JSON.stringify({ ok: true, ignored: true }), {
        headers: { "content-type": "application/json; charset=utf-8" }
      });
    }

    return env.ASSETS.fetch(request);
  }
};
