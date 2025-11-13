package util

import (
	"time"

	"github.com/jackc/pgx/v5/pgtype"
)

// TextToString safely unwraps pgtype.Text into a plain string.
func TextToString(t pgtype.Text) string {
	if !t.Valid {
		return ""
	}
	return t.String
}

// TimeToRFC3339 converts pgtype.Timestamptz to RFC3339 string in UTC.
func TimeToRFC3339(ts pgtype.Timestamptz) string {
	if !ts.Valid {
		return ""
	}
	return ts.Time.UTC().Format(time.RFC3339)
}
