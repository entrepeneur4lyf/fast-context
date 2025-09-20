package main

import (
	"os"

	"github.com/fast-context/go-sdk/cli"
)

func main() {
	app := cli.NewCLI()
	if err := app.Execute(); err != nil {
		os.Exit(1)
	}
}