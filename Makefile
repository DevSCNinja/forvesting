VESTING_SCHEDULE_PROGRAM_ID=$(shell eval solana-keygen pubkey ./vesting_schedule-keypair.json)
VESTING_SCHEDULE_PROGRAM_DEPLOY_ACCOUNT=$(shell eval solana-keygen pubkey ./keypair.json)
include ./.env

.PHONY: build deploy build-client run listen deploy-new

build:
	mkdir -p ./target/deploy/
	cp vesting_schedule-keypair.json ./target/deploy/
	anchor build -p vesting_schedule --provider.cluster $(DEPLOYMENT_CLUSTER) --provider.wallet ./keypair.json

deploy-new:
	anchor deploy

deploy:
	echo $(VESTING_SCHEDULE_PROGRAM_ID)
	anchor upgrade ./target/deploy/vesting_schedule.so --program-id $(VESTING_SCHEDULE_PROGRAM_ID) --provider.wallet ./keypair.json

start-validator:
	solana-test-validator

listen:
	solana logs $(VESTING_SCHEDULE_PROGRAM_ID)

airdrop:
	solana airdrop 5 $(VESTING_SCHEDULE_PROGRAM_DEPLOY_ACCOUNT) --url http://127.0.0.1:8899