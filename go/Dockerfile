# syntax=docker/dockerfile:1.7

# --- Build stage ---
FROM --platform=$BUILDPLATFORM golang:1.22-alpine AS build
WORKDIR /src

# Cache deps
COPY go/go.mod ./go.mod
RUN --mount=type=cache,target=/go/pkg/mod \
    go mod download

# Copy source
COPY go/ ./

# Build
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    CGO_ENABLED=0 GOOS=linux GOARCH=$TARGETARCH go build -o /out/messaging-go ./cmd/server

# --- Runtime stage ---
FROM gcr.io/distroless/base-debian12:nonroot
WORKDIR /app
COPY --from=build /out/messaging-go /app/messaging-go
EXPOSE 8080
USER nonroot:nonroot
ENTRYPOINT ["/app/messaging-go"]
