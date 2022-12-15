package extractor

import (
	"io"
	"time"

	"github.com/influxdata/go-syslog/v3"

	"git.sr.ht/~paul/logsnarf-go/internal/parser"
)

// LogData fuckoff
type LogData struct {
	Timestamp time.Time
	Appname   string
	Procid    string
	Pairs     map[string]string
}

// Parse fuckoff go
func Parse(buf io.Reader) (results []syslog.Result) {
	acc := func(res *syslog.Result) {
		results = append(results, *res)
	}

	// func NewParser(opts ...syslog.ParserOption) syslog.Parser { ... }
	parser := parser.NewParser(syslog.WithBestEffort(), syslog.WithListener(acc))
	parser.Parse(buf)

	return results
}
