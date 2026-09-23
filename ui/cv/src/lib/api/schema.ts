// The wire, as the protocol's transcoder renders tbd.cv.v1 (proto field
// names, every field present, timestamps as RFC 3339 strings, bytes as
// base64). Change proto/tbd/cv/v1/cv.proto and this file together.
import { z } from "zod";

/** `GET /v1/me` on the protocol: the principal Envoy verified. */
export const Me = z.object({
  subject: z.string(),
  kind: z.string(),
  client_id: z.string().nullable().optional(),
  org: z.string().nullable().optional(),
  scopes: z.array(z.string()),
  role: z.string().nullable().optional(),
  email: z.string().nullable().optional(),
  name: z.string().nullable().optional(),
});
export type Me = z.infer<typeof Me>;

export const AccessState = z.object({
  status: z.string(),
  email: z.string(),
  name: z.string(),
  note: z.string(),
  requested_at: z.string(),
  decided_at: z.string(),
  notified_at: z.string(),
  notifications: z.boolean(),
});
export type AccessState = z.infer<typeof AccessState>;

export const AccessResponse = z.object({ state: AccessState });

export const DownloadResponse = z.object({
  content_type: z.string(),
  pdf: z.string(),
  filename: z.string(),
});

export const AccessRequest = z.object({
  id: z.string(),
  subject: z.string(),
  email: z.string(),
  name: z.string(),
  note: z.string(),
  status: z.string(),
  requested_at: z.string(),
  decided_at: z.string(),
  decided_by: z.string(),
  notified_at: z.string(),
  downloads: z.number(),
  last_download_at: z.string(),
});
export type AccessRequest = z.infer<typeof AccessRequest>;

export const ListRequestsResponse = z.object({ requests: z.array(AccessRequest) });
export const DecideResponse = z.object({ request: AccessRequest });
