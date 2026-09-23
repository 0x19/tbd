# ui/cv

The gated site behind `cv.<domain>`: a signed-in visitor asks for the full CV, the
owner decides, an approved visitor downloads a PDF prepared for them. Two pages, one
API (`/v1/cv/`), no server of its own.

| Task                   | What                                                                                                     |
| ---------------------- | -------------------------------------------------------------------------------------------------------- |
| `mise run ui:cv:dev`   | `next dev` on :3005 against the local cluster's edge (`NEXT_PUBLIC_CV_TOKEN` from `mise run auth:token`) |
| `mise run ui:cv:check` | prettier, eslint, tsc                                                                                    |
| `mise run ui:cv:build` | the image, `ghcr.io/0x19/tbd-cv-ui:dev`                                                                  |

`docs/cv/README.md` describes the whole flow; `CLAUDE.md` here what to keep true.
