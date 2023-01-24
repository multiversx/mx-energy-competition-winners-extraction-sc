
ALICE="~/alice.pem"
PROXY=https://testnet-gateway.multiversx.com
CHAIN_ID=T
ADDRESS=$(mxpy data load --key=address-testnet)

deploy() {
    mxpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} --gas-limit=50000000 \
    --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="deploy-testnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="deploy-testnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-testnet --value=${ADDRESS}
    mxpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

addParticipantsAddresses() {
    ADDRESSES=()
    ADDRESSES_PER_TX=200
    while read address; do
        if [ $ADDRESSES_PER_TX -gt 0 ]
        then
            # appendBech32 ${address}
            ADDRESSES+=(${address})
            ((ADDRESSES_PER_TX=ADDRESSES_PER_TX-1))

        else
            mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=300000000 \
            --function="addParticipantsAddresses" --arguments ${ADDRESSES[@]} --send --proxy=${PROXY} --chain=${CHAIN_ID}
            ADDRESSES=()
            ADDRESSES_PER_TX=200
            sleep 6
        fi
    done < ./interaction/addresses.txt
    sleep 6
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=300000000 \
    --function="addParticipantsAddresses" --arguments ${ADDRESSES[@]} --send --proxy=${PROXY} --chain=${CHAIN_ID}     
}

extractWinners() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=300000000 \
    --function="extractWinners" --arguments 50 --send --proxy=${PROXY} --chain=${CHAIN_ID}     
}

distributeRewards() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=300000000 \
    --function="distributeRewards" --value 2663000000000000000 --send --proxy=${PROXY} --chain=${CHAIN_ID}     
}

appendBech32() {
    if [ -z "$ADDRESSES" ]
    then
        ADDRESSES="$1"
    else
        ADDRESSES="$ADDRESSES@$1"
    fi

}