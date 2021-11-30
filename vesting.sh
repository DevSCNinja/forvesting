# set +x

# solana-keygen new -o /root/.config/solana/id.json --no-bip39-passphrase
solana config set -k ./keypair.json
cp keypair.json /root/.config/solana/id.json
solana config set --url http://localhost:8899/
solana-test-validator > /dev/null 2>&1 &
sleep 10
solana airdrop 5000 BSKmmWSyV42Pw3AwZHRFyiHpcBpQ3FyCYeHVecUanb6y
mkdir -p ./target/deploy/
cp vesting_schedule-keypair.json ./target/deploy/
make build
make deploy-new
npx ts-mocha --parallel --jobs="$(find tests -type f -name 'tests_*.ts' | wc -l)" 'tests/tests_*.ts'