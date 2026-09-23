// The redaction check for the site's lab: what a public RFC or study may not
// carry. Every rule is a regular expression over one line of the page, with
// the `[REDACTED: reason]` markers already reduced to their reasons (the
// reason renders on hover, so it is public text and is checked too).
//
// The rules infer hosts, addresses, ports, paths, secrets and cluster names;
// `docs/lab/redaction.json` lists the nouns they cannot (resource names).
// A hit is `path:line [rule] "text"`, and the build stops.

export type Rule = { id: string; re: RegExp; why: string };

export const RULES: readonly Rule[] = [
  {
    id: "domain",
    // A subdomain of the company's domains. The bare domain and `www.` are the site itself.
    re: /\b(?!www\.)[a-z0-9-]+\.(?:inorbit\.hr|proximity\.is)\b/i,
    why: "an internal host name",
  },
  {
    id: "ipv4",
    re: /\b(?:\d{1,3}\.){3}\d{1,3}\b/,
    why: "an address",
  },
  {
    id: "port",
    // `:1024` to `:65535` after a host or on its own; `:80`, `12:30` and years pass.
    re: /:(?:102[4-9]|10[3-9]\d|1[1-9]\d{2}|[2-9]\d{3}|[1-5]\d{4}|6[0-4]\d{3}|65[0-4]\d{2}|655[0-2]\d|6553[0-5])\b/,
    why: "a port",
  },
  {
    id: "path",
    re: /(?:^|[\s"'`(=])(?:\/mnt\/|\/opt\/|\/home\/|devops\/|configs\/|\.env\b)/,
    why: "a path into the machine or the deployment",
  },
  {
    id: "secret",
    re: /\b(?:Secret|secretKeyRef|x-jwt|token_bucket|envoy\.yaml|jwt_authn|oauth2)\b/,
    why: "the shape of the gate",
  },
  {
    id: "k8s",
    re: /\b[a-z0-9-]+\.[a-z0-9-]+\.svc(?:\.cluster\.local)?\b|^\s*(?:kind|namespace):\s*\S+|\s-n\s+[a-z0-9-]+\b/,
    why: "a cluster name",
  },
  {
    id: "embed",
    // Raw HTML that loads something, or an image from another origin: the
    // legal page promises nothing from a third party.
    re: /<(?:script|iframe|link|img)\b|!\[[^\]]*\]\(https?:/i,
    why: "content loaded from elsewhere",
  },
];

export type Hit = { line: number; rule: string; text: string; why: string };

const escape = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

/** The inline markers reduced to their reasons, which are public. */
export function stripMarkers(line: string): string {
  return line.replace(/\[REDACTED:\s*([^\]]*)\]/g, "$1");
}

/**
 * Every hit in `lines` (front matter values and body, fences included) against
 * the rules and the denylist. `redactedFence` lines are the delimiters of a
 * ```redacted block and are skipped; the block's body (the reason) is checked.
 */
export function scan(lines: readonly string[], denylist: readonly string[]): Hit[] {
  const deny: Rule[] = denylist.map((w) => ({
    id: "denylist",
    re: new RegExp(`\\b${escape(w)}\\b`, "i"),
    why: `"${w}" names a piece of the deployment`,
  }));
  const hits: Hit[] = [];
  lines.forEach((raw, i) => {
    if (/^\s*```/.test(raw)) return;
    const line = stripMarkers(raw);
    for (const rule of [...RULES, ...deny]) {
      const m = rule.re.exec(line);
      if (m) hits.push({ line: i + 1, rule: rule.id, text: m[0].trim(), why: rule.why });
    }
  });
  return hits;
}
