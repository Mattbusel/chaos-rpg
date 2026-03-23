/**
 * CHAOS RPG — Daily Leaderboard Worker (Cloudflare Workers + KV)
 *
 * Deploy:
 *   1. Create a Cloudflare account (free tier is enough)
 *   2. Install wrangler: npm install -g wrangler
 *   3. wrangler login
 *   4. wrangler kv:namespace create LEADERBOARD
 *   5. Update wrangler.toml with the namespace id
 *   6. wrangler deploy
 *
 * Endpoints:
 *   POST /submit  — record a score
 *   GET  /scores?date=YYYY-MM-DD  — top 100 for a day
 *   GET  /health  — sanity check
 *
 * KV layout:
 *   key: "scores:YYYY-MM-DD"  →  JSON array of LeaderboardEntry (max 100, sorted by score desc)
 */

const MAX_ENTRIES = 100;
const CORS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type",
};

function json(data, status = 200) {
  return new Response(JSON.stringify(data), {
    status,
    headers: { "Content-Type": "application/json", ...CORS },
  });
}

function sanitize(s, max = 32) {
  if (typeof s !== "string") return "";
  return s.replace(/[^\w\s\-\.\/]/g, "").slice(0, max).trim();
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    // CORS preflight
    if (request.method === "OPTIONS") {
      return new Response(null, { headers: CORS });
    }

    // ── GET /health ──────────────────────────────────────────────────────────
    if (url.pathname === "/health") {
      return json({ ok: true, ts: Date.now() });
    }

    // ── GET /scores ──────────────────────────────────────────────────────────
    if (request.method === "GET" && url.pathname === "/scores") {
      const date = url.searchParams.get("date") || today();
      if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) {
        return json({ error: "invalid date" }, 400);
      }
      const raw = await env.LEADERBOARD.get(`scores:${date}`);
      const rows = raw ? JSON.parse(raw) : [];
      return json(rows);
    }

    // ── POST /submit ─────────────────────────────────────────────────────────
    if (request.method === "POST" && url.pathname === "/submit") {
      let body;
      try { body = await request.json(); } catch { return json({ error: "bad json" }, 400); }

      const date  = (typeof body.date === "string" && /^\d{4}-\d{2}-\d{2}$/.test(body.date))
                    ? body.date : today();
      const entry = {
        name:  sanitize(body.name  || "Anon"),
        class: sanitize(body.class || "Unknown"),
        floor: Math.max(0, Math.min(99999, parseInt(body.floor) || 0)),
        score: Math.max(0, parseInt(body.score) || 0),
        kills: Math.max(0, parseInt(body.kills) || 0),
        seed:  String(body.seed || "0").slice(0, 20),
        won:   body.won === true,
        ts:    Date.now(),
      };

      const key = `scores:${date}`;
      const raw = await env.LEADERBOARD.get(key);
      let rows = raw ? JSON.parse(raw) : [];

      // De-duplicate by name (keep best score)
      const existing = rows.findIndex(r => r.name === entry.name);
      if (existing !== -1) {
        if (entry.score > rows[existing].score) {
          rows[existing] = entry;
        }
      } else {
        rows.push(entry);
      }

      // Sort descending by score, cap at MAX_ENTRIES
      rows.sort((a, b) => b.score - a.score);
      rows = rows.slice(0, MAX_ENTRIES);

      // Assign ranks
      rows.forEach((r, i) => { r.rank = i + 1; });

      // Store with 48h TTL (daily data stays for 2 days)
      await env.LEADERBOARD.put(key, JSON.stringify(rows), { expirationTtl: 172800 });

      // Find this submitter's rank
      const myRank = (rows.findIndex(r => r.name === entry.name) + 1) || MAX_ENTRIES;
      return json({ ok: true, rank: myRank });
    }

    return json({ error: "not found" }, 404);
  },
};

function today() {
  return new Date().toISOString().slice(0, 10);
}
