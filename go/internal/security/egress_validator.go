package security

import (
    "errors"
    "net"
    "net/url"
    "strings"
)

// Validator validates outbound URLs against an allowlist and blocks private/internal ranges.
type Validator struct {
    allowlist []string // host suffixes or exact hosts
}

func NewValidator(allowlist []string) *Validator {
    norm := make([]string, 0, len(allowlist))
    for _, h := range allowlist {
        h = strings.TrimSpace(strings.ToLower(h))
        if h != "" {
            norm = append(norm, h)
        }
    }
    return &Validator{allowlist: norm}
}

var privateCIDRs = []string{
    "10.0.0.0/8",
    "172.16.0.0/12",
    "192.168.0.0/16",
    "127.0.0.0/8",
    "169.254.0.0/16",
    "::1/128",
    "fc00::/7",
    "fe80::/10",
}

// IsAllowed returns nil if the URL is permitted.
func (v *Validator) IsAllowed(raw string) error {
    u, err := url.Parse(raw)
    if err != nil || u.Host == "" {
        return errors.New("invalid url")
    }
    host := u.Host
    if strings.Contains(host, ":") {
        host, _, _ = strings.Cut(host, ":")
    }
    lh := strings.ToLower(host)
    if !v.matchAllow(lh) {
        return errors.New("host not in allowlist")
    }
    ips, err := net.LookupIP(lh)
    if err != nil || len(ips) == 0 {
        return errors.New("host resolve failed")
    }
    // Pre-parse CIDRs
    parsed := make([]*net.IPNet, 0, len(privateCIDRs))
    for _, c := range privateCIDRs {
        _, n, _ := net.ParseCIDR(c)
        parsed = append(parsed, n)
    }
    for _, ip := range ips {
        for _, n := range parsed {
            if n.Contains(ip) {
                return errors.New("resolved ip in private range")
            }
        }
    }
    return nil
}

func (v *Validator) matchAllow(host string) bool {
    for _, a := range v.allowlist {
        if host == a || strings.HasSuffix(host, "."+a) {
            return true
        }
    }
    return false
}
 
