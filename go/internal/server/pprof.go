package server

import (
    "net/http"
    "net/http/pprof"

    "github.com/go-chi/chi/v5"
)

// MountPprof mounts net/http/pprof handlers under the provided base path.
// Example path: /debug/pprof
func MountPprof(r *chi.Mux, base string) {
    r.Route(base, func(r chi.Router) {
        r.Get("/", pprofIndex)
        r.Get("/cmdline", pprof.Cmdline)
        r.Get("/profile", pprof.Profile)
        r.Get("/symbol", pprof.Symbol)
        r.Get("/trace", pprof.Trace)
        r.Get("/allocs", pprof.Handler("allocs").ServeHTTP)
        r.Get("/block", pprof.Handler("block").ServeHTTP)
        r.Get("/goroutine", pprof.Handler("goroutine").ServeHTTP)
        r.Get("/heap", pprof.Handler("heap").ServeHTTP)
        r.Get("/mutex", pprof.Handler("mutex").ServeHTTP)
        r.Get("/threadcreate", pprof.Handler("threadcreate").ServeHTTP)
    })
}

func pprofIndex(w http.ResponseWriter, r *http.Request) { pprof.Index(w, r) }
