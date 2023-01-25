
ALICE="~/alice.pem"
ALICE_ADDRESS="erd1qyu5wthldzr8wx5c9ucg8kjagg0jfs53s8nr3zpz3hypefsdd8ssycr6th"
PROXY=https://devnet-gateway.multiversx.com
CHAIN_ID=D
ADDRESS=$(mxpy data load --key=address-devnet)

deploy() {
    mxpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} --gas-limit=50000000 \
    --send --outfile="deploy-devnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="deploy-devnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="deploy-devnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-devnet --value=${ADDRESS}
    mxpy data store --key=deployTransaction-devnet --value=${TRANSACTION}

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
            ADDRESSES+=(${address})
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

depositRewards() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=30000000 \
    --function="depositRewards" --value 292000 --send --proxy=${PROXY} --chain=${CHAIN_ID}     
}

distributeRewards() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=300000000 \
    --function="distributeRewards" --send --proxy=${PROXY} --chain=${CHAIN_ID}     
}

IDENTIFIER=MEX-dc289c
NONCE=0
AMOUNT=292000
depositMetaEsdtRewards() {
    echo ${ADDRESS}
    mxpy --verbose contract call ${ALICE_ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=300000000 \
    --function="MultiESDTNFTTransfer" --arguments ${ADDRESS} 1 str:${IDENTIFIER} ${NONCE} ${AMOUNT} str:"depositRewards" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}     
}

upgrade() {
    erdpy --verbose contract upgrade ${ADDRESS} --project=${PROJECT} --recall-nonce --pem=${ALICE} \
    --gas-limit=25000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

appendBech32() {
    if [ -z "$ADDRESSES" ]
    then
        ADDRESSES="$1"
    else
        ADDRESSES="$ADDRESSES@$1"
    fi

}