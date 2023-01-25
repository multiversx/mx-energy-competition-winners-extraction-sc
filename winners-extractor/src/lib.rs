#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use multiversx_sc_modules::ongoing_operation::{
    CONTINUE_OP, DEFAULT_MIN_GAS_TO_SAVE_PROGRESS, STOP_OP,
};


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
    pub per_user: BigUint<M>,
    pub current_number: usize,
}

/// An empty contract. To be used as a template when starting a new contract from scratch.
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
    #[endpoint(distributeRewards)]
    fn distribute_rewards(&self, mut num_winners: u32) -> OperationCompletionStatus {
        let payment = self.call_value().egld_or_single_esdt();
        let participants = self.participants();
        let caller = self.blockchain().get_caller();
        let mut pending_distribution = self.get_current_distribution(
            payment.token_identifier.clone(), 
            payment.amount.clone(),
            num_winners,
        );
        require!(self.participants_left() >= num_winners as usize, "Less participants left than users to distribute");
        require!(&pending_distribution.per_user * num_winners == payment.amount.clone(), "Invalid value sent");
        let run_result = self.run_while_it_has_gas(DEFAULT_MIN_GAS_TO_SAVE_PROGRESS, || {
            if pending_distribution.current_number == participants.len() {
                self.cancel_distribution();
                return STOP_OP;
            }

            if num_winners == 0 {
                self.pending_distribution().set(&pending_distribution);
                return STOP_OP;
            }

            let addr = participants.get(pending_distribution.current_number + 1);
            if payment.token_identifier.is_egld() {
                self.send().direct_egld(&addr, &pending_distribution.per_user);
            } else {
                self.send().direct_esdt(&addr, &payment.token_identifier.clone().unwrap_esdt(), payment.token_nonce, &pending_distribution.per_user);
            }
            pending_distribution.current_number += 1;
            num_winners -= 1;

            CONTINUE_OP
        });

        if run_result == OperationCompletionStatus::InterruptedBeforeOutOfGas {
            self.pending_distribution().set(&pending_distribution);
            if payment.token_identifier.is_egld() {
                self.send().direct_egld(&caller, &(&pending_distribution.per_user * num_winners));
            } else {
                self.send().direct_esdt(&caller, &payment.token_identifier.clone().unwrap_esdt(), payment.token_nonce, &(&pending_distribution.per_user * num_winners));
            }
        }

        run_result
    }

    #[only_owner]
    #[endpoint(cancelDistribution)]
    fn cancel_distribution(&self) {
        self.pending_distribution().clear();
    }

    #[view(participantsLeft)]
    fn participants_left(&self) -> usize {
        let participants = self.participants();
        let pending_distribution_mapper = self.pending_distribution();
        if pending_distribution_mapper.is_empty() {
            return participants.len()
        } else {
            participants.len() - &pending_distribution_mapper.get().current_number
        }
    }

    fn get_current_distribution(&self, token: EgldOrEsdtTokenIdentifier, total_amount: BigUint<Self::Api>, num_winners: u32) -> PendingDistribution<Self::Api> {
        let pending_distribution_mapper = self.pending_distribution();
        let per_user = total_amount / num_winners;
        require!(per_user > 0u32, "Distribute amount cannot be zero");
        if pending_distribution_mapper.is_empty() {
            PendingDistribution {
                payment: token,
                per_user: per_user,
                current_number: 0,
            }
        } else {
            let distribution = pending_distribution_mapper.get();
            require!(distribution.payment == token, "A different distribution is in progress");
            distribution
        }
    }

    #[endpoint(extractWinners)]
    fn extract_winners(&self, num_winners: u64) -> MultiValueEncoded<ManagedAddress> {
        let mut rng = RandomnessSource::default();
        let mut participants = self.participants();
        let mut winners = MultiValueEncoded::new();
        for _ in 0..num_winners {
            let winner_index = rng.next_usize_in_range(0, participants.len());
            let winner = participants.get(winner_index);
            winners.push(winner.clone());
            participants.swap_remove(winner_index);
        }
        winners
    }

    #[view(getPendingDistribution)]
    #[storage_mapper("pending_distribution")]
    fn pending_distribution(&self) -> SingleValueMapper<PendingDistribution<Self::Api>>;

    #[view(getParticipants)]
    #[storage_mapper("participants")]
    fn participants(&self) -> VecMapper<ManagedAddress>;
}
