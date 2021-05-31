package cmd

import (
	"bufio"
	"os"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/spf13/cobra"

	"git.sr.ht/~paul/logsnarf-go/internal/handler"
)

var (
	token    string
	filename string

	parseCmd = &cobra.Command{
		Use:   "parse",
		Short: "Parse metrics from a file on disk",
		RunE:  runCmd,
	}
)

func init() {
	log.Logger = log.Output(zerolog.ConsoleWriter{Out: os.Stderr})

	rootCmd.AddCommand(parseCmd)

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// parseCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// parseCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")

	parseCmd.Flags().StringVarP(&token, "token", "t", "", "Authentication Token")
	parseCmd.MarkFlagRequired("token")

	parseCmd.Flags().StringVarP(&filename, "filename", "f", "", "File to parse")
	parseCmd.MarkFlagRequired("filename")
}

func runCmd(cmd *cobra.Command, args []string) error {
	file, err := os.Open(filename)
	if err != nil {
		log.Error().Err(err).Msg("")
		return err
	}
	defer file.Close()

	buf := bufio.NewReader(file)

	err = handler.Handle(token, buf)
	if err != nil {
		log.Error().Err(err).Msg("")
		return err
	}

	return nil
}
