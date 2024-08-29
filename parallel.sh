#!/bin/bash

# Define the list of escrow owners

accounts=(
#"9gw9VUBsbBfjcqwoR5tUvxwfoCzjonoR2UwSPi2qyAYe"
#"H7UJp5DXapdzBwEf1v3BiVeUkDeySoQuSQD2fkG5Qx4A"
#"2b5YJcaLBRgiB6kQ9YHiPp1hGehuopGbQFQsomiPpzAJ"
#"DLSHcmTCKhFqG4NXbfzHdX6fXzY1ofmZWLXFcDVGVVH6"
#"FL5CBUXqcZc39skbiZdAogAB96symx9sRcsSPksruB2X"
#"GtZx8qh726SMeyLBde7zBH5SdkLv9wHQBRxXZmGCyncz"
#"APWFSDwY2v8xbVZhLkrKDiwJSNQrtNSwP5R3ZkAFRoAE"
#"7KWPGsT8LioJpuG5Lh1DbJepWTLBs4u2ezYgdxpyssLi"
#"7xyBo5HQqrzxuHjud7SCD4M57EonpEF1DKUFRGssNEDb"
#"H8avLeidfpcnYKtcykCcwyUPeDR8yAVDyr2d4sUk4Jqs"
#"2Qy4xGR9zjwzSFvXZrMwCHmkV42KLGDoKCuMW2fH5hqK"
#"2iWWL2Tsmp7tTMaGs47wsQgTAoHpqJ87q6X7tHjd6eGA"
#"G1N7agkSbSEuNtW9YRwrySUMmtR5JFUUmQu31G4j4nTj"
#"EbAqEsrUXth52i4jyTeMLQNAdrHWqWB9QuS4JK83ZLhg"
#"9j8aB79bqQ2o5NRjFw1Q8CgHhqUvBCzNX12GQiRieGGZ"
#"BCwCpeABMuENfyX8C8dVW24fjByz1ZdoUM7ASYk8BnqE"
#"BMi9pBV2xoDeYENyRghqayx1YHxtMKiG6y8rf76JhyYt"
#"7JSqs1zM9aapNFVZePi8C7h2eD5NmZm4xzgjyQQX32tx"
#"98ya1KG3ic3BWGubMLjrfaLLCK3Cucy7niPRhbrBbqXw"
#"8QquE1jBq9REPiqnnw8om4xi6jLefEobCusYvUs18uFv"
#"GgPR2wwTFxguXyTeMmtrhipfv4A8Y3vdPX7RLQNa1zJ3"
#"Eu7qycf5roJNB2R5AVbQxmakLTNW71QX1qLmisaczXna"
#"Da7sr5QRrNG1VHpeut2HTc68591BAdYeMPScNRLVbVq8"
#"6wChdchEiawikhbALSBghEeih5iJcHSZYLT42xUtMxcc"
#"DU4EJnA2z5HbnW8CzAuad9mVoe3cGycjU2HgAksTWVPp"
#"GKsndoQnLgrSeR76g7VbctM9A9MfnoUoRsjUYGSf9wGJ"
#"5R4Tw4jSePJ65Aqb61uhvP6BdQLyzZjjMiTbEccH2BcF"
#"EkkGT9eVeKvv7NHVYySYtzyTuqEYRShYw27wxgDdNvFB"
#"6CiT4o1XgXwBwy3p2wVALXVyLNkjctxJB1GxHqPpuTCh"
#"HD8yVzkYbRVvA7PuWo3rzgM91rumhmRLjZr8GzfGt7TQ"
#"Egq6SXZsUCgGonaAtn33TMJ7JBQT62wFwQTKybyL79DY"
#"3zx59Tics3Mh2rp1kCYwMKwAK6FCEppteAjB6zvdt1kB"
#"BeurG5Vu2RhsH3KEKy74fPqs38RCiMdm473zJDq6hzhR"
#"J2EZPYQ9VUdfBdXBNo6ZsCWgpZysNTazcTr66kS6fsab"
#"UuGEwN9aeh676ufphbavfssWVxH7BJCqacq1RYhco8e"
#"EN2CV9nCnH9nBF9GyGYG9B3haNriNBkrPo8jF4c6mzUi"
#"9DWREMQtcKKcZRzqBi4kWKp61TW9MhASCVE9biCX7tvD"
#"8moDwyGgWV9a2THBC3VSrBx5SLHgT1dggFfVenqrDcq4"
#"3rukEvuPaVxmr85BxSuoYs263KA1u6JCL2q72gCzBQqk"
#"Bff7doQpyfo7tjJcktt45eDvtbKjmgj9T9W9zzJHrGTk"
#"8H7AQ2NuWpUgB6huu9Jhho68mPS32kGNHdEvPEXQEkWT"
#"iJaQGNcWoZ3Yi4UfWqxiEbENj7fi8aj9X7QgVfGdakb"
#"9uPQ7AWTdLrurp2aD2Ww7MjZfBHtxFFoc437WrnWizCJ"
#"3feUnWTeUUH8w3kYvfgotYVE4nK6JwkaG9875bMaU4Ym"
#"3SVeFPSRrhWbZFw2vvv8hf5H5t71gL7wLRQdDnUGw4WV"
#"9VbwqRfJQ22dYZktJSokJ8gH6KXAknaUx3Df9KvDkMXq"
#"HTHmunfvDUhCr3AWwccbr7dPzm2sKVhKGnGcgGDZKHpG"
#"9jRVHbqvPaQKrXbWcYv9fmPL1fRnV61u4iYGT5bVdTXP"
#"GVQzKmFPJDrJT1pBYiT7KhfhMqvmU6yQ6SAhQKR7roaj"
#"HTtpTSLqSb6eAkc1FdT3igh4N7edWYMYakxbFbhuSBE5"
#"D1PSXTiweEPKLece65KR5V3BFfxF6NerQ2fn7qzAdMaM"
#"6FGSQGg88XJZceQv6b6aXvtEYJpzm46uV77Q4GUCgBRU"
#"4Dos4bnCNg5dmynDS1K8dDGsBTpCm5zdv4t7Y9Dgp9LW"
#"82vwhbqGYELjG1T162UQi9RPR5KisnKVgwbap6czHoXn"
#"2K6dkYdHssrZcwJcwRDTPmecgx2zw1b8McJ67ttYCwXp"
#"GJcw7aqV4jtXLS6pbpoU8RrzFgmKQLUM6XjEr41nLy3N"
#"AxTKvBsTfbhh95kAF7nHnE6SU1R9ngLscVr57y7PwtA2"
#"jSVjHiMcpXq2Bxr1DnW9qvxkVWLkMcDqBcguxFgiKKY"
#"BLAtrzKG8RSkgpCdMHHtTrd6pWRAgwKfhNGx1wTPKbTm"
#"GNa9E2dWP8e2fg23pRJoiCvJm5cbzomm94YzotpcRh7A"
#"3nGH1y9NVuirmxV5djUpYoPQnw1PGZvvkWH816kQffZW"
#"2m1Heosmm6nyJ77zjyTUswmaVZH3Wk1S2iZdkHHk3Ht7"
#"9uGKKCCtEPMYVvDbt83PATySwVDHpvke3CQMdzZWTVX3"
#"HFUex3SnvL8QvXFZq2tkxm529xxofNzgAiPvaXx1zCNS"
#"7h7PW3Fhp2izdgzh3VZ34Z2p9PRPeTBAo7ajr6Ha8eRT"
#"5Wn91tMYnqEDEiFkW4Tm3Ph82EUuA22abJp2nHv9vTJM"
#"HriwdkgqDHzvwxZP5fNVwt6RcRANztuNE9oEUFVqnRcF"
#"6cix41uBpzVKrGZWYkuGXuoqf3MFEc5q6rMNW62dqMXK"
#"3cSCu1bfWE1Eb6QHZD4X78qwue5pgVot58KJGoED6H3T"
#"C2CtVcKBypvr3wW5UYHV7FE9vaDZpdmNrq8MUCKcrsPg"
#"CoQCdvnweWDwWieTjD7uTNEQErSJuKGge8XtpsD1WCyG"
#"91NjPFfJxQw2FRJvyuQUQsdh9mBGPeGPuNavt7nMLTQj"
#"6BnTKZRNvNxe9pthVuVNkLoYNkW4wWyY6HzqR23te8g5"
"8jfJaDSuwovJ8qB32JgHYCu6sa8EzKn3GeJ5ZfqWkEM3"
#"AD47mvwEQPmoTqcrwQ8fzhqYPEGotWSeZrstLcZgcsqb"
#"CNJXoteJxRft3WaY78AuT7LXvCnqyX4NHR9kmfzWcUDW"
#"EaEfbpdVDX79mRfDtjPouoXn4weTnpUzme9vPuxbFkYn"
#"6A4g69uzSeSmyRbH7kL5rVcYFFTB7tUkdrAv35JGT1Jm"
#"3zttTwxdBtCLTNyc8NKXKwaq4Rx8QMAdQTiiUZ4es36w"
#"5XDN5ecaoQUkJXaga2i5xyxtV8apvwzdkSFHHVsMZDhu"
#"9Q5CwKDVfnBTyEe6i6qnqKfESJBFkKCW8KqqofkSggWE"
#"CArep5EtJv8uuRpdntkAYKXmSx9AWWWK9PAsQuarUq4X"
#"FeEUCNr75QxGUkHSdtom3LdEzpgQwr7YmNxJVqSwTsCG"
#"65U66fcYuNfqN12vzateJhZ4bgDuxFWN9gMwraeQKByg"
)

max_parallel_jobs=5
current_jobs=0

start_time=$(date +%s)
# Execute the command for each account in parallel
for account in "${accounts[@]}"
do
     echo "Processing account: $account"
    ./target/release/vote-market-manager execute-votes epoch_125_vote_info2024-08-24-15_53.json epoch_125_weights2024-08-24-15_56.json -k ~/.config/solana/script-authority.json -e "$account" &
        current_jobs=$((current_jobs + 1))

        # If the number of current jobs equals the maximum allowed, wait for them to finish
        if [ "$current_jobs" -ge "$max_parallel_jobs" ]; then
            wait -n
            # Decrease the count of current jobs after one completes
            current_jobs=$((current_jobs - 1))
        fi
done

# Wait for all background processes to finish
wait

end_time=$(date +%s)

elapsed_time=$((end_time - start_time))

echo "All processes completed in $elapsed_time secondS."
