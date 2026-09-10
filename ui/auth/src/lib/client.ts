"use client";

import { Configuration, FrontendApi } from "@ory/client-fetch";

// Same-origin Kratos public API (Envoy routes /self-service and /sessions here).
export const frontend = new FrontendApi(
  new Configuration({
    basePath: typeof window === "undefined" ? "" : window.location.origin,
    credentials: "include",
  }),
);
