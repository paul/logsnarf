package handler

import (
	"bufio"

	"github.com/rs/zerolog/log"

	"git.sr.ht/~paul/logsnarf-go/internal/credentials"
	"git.sr.ht/~paul/logsnarf-go/internal/extractor"
)

// Handle Parse buffer
func Handle(token string, buf *bufio.Reader) error {

	creds, err := credentials.Fetch(token)
	if err != nil {
		log.Error().Err(err).Msg("problem fetching credentials")
		return err
	}

	log.Debug().Interface("Creds", creds).Msg("")

	_ = extractor.Extract(buf)

	return nil
}
