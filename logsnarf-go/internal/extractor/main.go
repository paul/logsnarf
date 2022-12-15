package extractor

import (
	"io"
	"time"

	"github.com/rs/zerolog/log"
)

// Metric fuckoff go
type Metric struct {
	Name      string
	Tags      map[string]string
	Fields    map[string]interface{}
	Timestamp time.Time
}

// Extract fuckoff go
func Extract(buf io.Reader) (metrics []Metric) {
	data := Parse(buf)

	log.Debug().Interface("Creds", data).Msg("")

	return nil
}
