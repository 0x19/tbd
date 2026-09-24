# The sandbox's Go toolchain (docs/sandbox/README.md, RFC 0010). Built locally by
# `mise run sandbox:images`, run only by sandboxd under gVisor, never pushed.
# Pinned by digest: a new toolchain is a deliberate change.
FROM golang:1.27.1@sha256:3680233e3204827fbdc66088528ae6d4b3d034f51d03a99d454f6de034888244

# Offline, no C, the toolchain that is installed and no other.
ENV CGO_ENABLED=0 GOTOOLCHAIN=local GOPROXY=off GOFLAGS=-trimpath GOENV=off \
    GOCACHE=/opt/gocache

# The standard library built once, into a cache the sandbox reads but cannot
# write (the image is read-only at run time): a program's build is then about a
# second rather than the twenty-four a cold one takes on one processor.
RUN go build std && chmod -R a+rX /opt/gocache

# Nothing runs as root in the sandbox; the recipe also sets the user.
USER 65534:65534
WORKDIR /work
