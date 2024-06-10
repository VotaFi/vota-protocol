solana-test-validator --reset --clone-upgradeable-program GaugesLJrnVjNNWLReiw3Q7xQhycSBRgeHGTMDUaX231 \
	--clone-upgradeable-program LocktDzaV1W2Bm9DeZeiyz4J9zs4fRqNiYqQyracRXw -um\
	--account-dir ./test-accounts > /dev/null 2>&1 &
sleep 5
anchor test --skip-local-validator
pkill -f solana-test-validator