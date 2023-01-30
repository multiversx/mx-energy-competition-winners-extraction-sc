#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use multiversx_sc_modules::ongoing_operation::{
    CONTINUE_OP, DEFAULT_MIN_GAS_TO_SAVE_PROGRESS, STOP_OP,
};

const MAX_USER_DISTRIBUTION: usize = 500;

#[derive(
    ManagedVecItem,
    TopEncode,
    TopDecode,
    NestedEncode,
    NestedDecode,
    TypeAbi,
    Clone,
    PartialEq,
    Debug,
)]
pub struct PendingDistribution<M: ManagedTypeApi> {
    pub payment: EgldOrEsdtTokenIdentifier<M>,
    pub payment_nonce: u64,
    pub per_user: BigUint<M>,
    pub current_number: usize,
}

#[multiversx_sc::contract]
pub trait ExtractWinnersContract: multiversx_sc_modules::ongoing_operation::OngoingOperationModule {
    #[init]
    fn init(&self) {}

    #[only_owner]
    #[endpoint(addParticipantsAddresses)]
    fn add_participants_addresses(&self, participants_addr: MultiValueEncoded<ManagedAddress>) -> usize {
        let mut participants = self.participants();
        for addr in participants_addr {
            let _ = participants.push(&addr);
        }
        participants.len()
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(depositRewards)]
    fn deposit_rewards(&self) {
        require!(self.pending_distribution().is_empty(), "A distribution is in progress");
        let payment = self.call_value().egld_or_single_esdt();
        let participants = self.participants();
        let num_participants = participants.len() as u32;
        let per_user = &payment.amount / num_participants;
        require!(per_user > 0u32, "Distribute amount cannot be zero");
        require!(&per_user * num_participants == payment.amount, "Invalid value sent");
        self.pending_distribution().set(PendingDistribution {
            payment: payment.token_identifier,
            payment_nonce: payment.token_nonce,
            per_user,
            current_number: 0,
        });
    }

    #[only_owner]
    #[endpoint(distributeRewards)]
    fn distribute_rewards(&self) -> OperationCompletionStatus {
        let mut max_users_per_step = MAX_USER_DISTRIBUTION;
        let participants = self.participants();
        let mut pending_distribution = self.pending_distribution().get();
        let run_result = self.run_while_it_has_gas(DEFAULT_MIN_GAS_TO_SAVE_PROGRESS, || {
            if pending_distribution.current_number == participants.len() {
                self.pending_distribution().clear();
                return STOP_OP;
            }

            if max_users_per_step == 0 {
                self.pending_distribution().set(&pending_distribution);
                return STOP_OP;
            }

            let addr = participants.get(pending_distribution.current_number + 1);
            self.send().direct(&addr, &pending_distribution.payment, pending_distribution.payment_nonce, &pending_distribution.per_user);
            pending_distribution.current_number += 1;
            max_users_per_step -= 1;

            CONTINUE_OP
        });

        if run_result == OperationCompletionStatus::InterruptedBeforeOutOfGas {
            self.pending_distribution().set(&pending_distribution);
        }

        run_result
    }

    #[endpoint(extractWinners)]
    fn extract_winners(&self, num_winners: u64) -> MultiValueEncoded<ManagedAddress> {
        require!(self.pending_distribution().is_empty(), "A distribution is in progress");
        let mut rng = RandomnessSource::default();
        let mut participants = self.participants();
        let mut winners = MultiValueEncoded::new();
        for _ in 0..num_winners {
            let winner_index = rng.next_usize_in_range(0, participants.len());
            let winner = participants.get(winner_index);
            winners.push(winner.clone());
            participants.swap_remove(winner_index);
        }
        for winner in winners.to_vec().iter() {
            let _ = participants.push(&winner);
        }
        winners
    }

    #[only_owner]
    #[endpoint(cancelDistribution)]
    fn cancel_distribution(&self) {
        require!(!self.pending_distribution().is_empty(), "No distribution in progress");
        let pending_distribution = self.pending_distribution().get();
        let amount_left = self.blockchain().get_sc_balance(&pending_distribution.payment, pending_distribution.payment_nonce);
        
        self.send().direct(
            &self.blockchain().get_caller(),
            &pending_distribution.payment,
            pending_distribution.payment_nonce,
            &amount_left,
        );
        self.pending_distribution().clear();
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(distributeRewardsSingle)]
    fn distribute_rewards_single(&self, receiver: ManagedAddress) {
        let payment = self.call_value().egld_or_single_esdt();
        self.send().direct(&receiver, &payment.token_identifier, payment.token_nonce, &payment.amount);
    }

    #[view(participantsLeft)]
    fn participants_left(&self) -> usize {
        let participants = self.participants();
        let pending_distribution_mapper = self.pending_distribution();
        if pending_distribution_mapper.is_empty() {
            participants.len()
        } else {
            participants.len() - &pending_distribution_mapper.get().current_number
        }
    }

    #[view(getPendingDistribution)]
    #[storage_mapper("pending_distribution")]
    fn pending_distribution(&self) -> SingleValueMapper<PendingDistribution<Self::Api>>;

    #[view(getParticipants)]
    #[storage_mapper("participants")]
    fn participants(&self) -> VecMapper<ManagedAddress>;
}
