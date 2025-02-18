
install-cometbft:
	GOBIN=/usr/local/bin/ go install github.com/cometbft/cometbft/cmd/cometbft@v1.0.1

prune:
	rm -rf ~/.cometbft
