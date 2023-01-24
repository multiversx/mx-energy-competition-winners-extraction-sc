#![no_std]

multiversx_sc::imports!();

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait ExtractWinnersContract {
    #[init]
    fn init(&self) {}

    #[only_owner]
    #[endpoint(addParticipantsAddresses)]
    fn add_participants_addresses(&self, participants_addr: MultiValueEncoded<ManagedAddress>) {
        let mut participants = self.participants();
        for addr in participants_addr {
            let _ = participants.push(&addr);
        }
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(distributeESDTRewards)]
    fn distribute_esdt_rewards(&self) {
        let payment = self.call_value().single_esdt();
        let participants = self.participants();
        let per_user = payment.amount / participants.len() as u32;
        require!(per_user > 0u32, "Distribute amount cannot be zero");
        for addr in participants.iter() {
            self.send().direct_esdt(&addr, &payment.token_identifier, payment.token_nonce, &per_user);
        }
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(distributeRewards)]
    fn distribute_rewards(&self) {
        let payment_amount = self.call_value().egld_value();
        let participants = self.participants();
        let per_user = payment_amount / participants.len() as u32;
        require!(per_user > 0u32, "Distribute amount cannot be zero");
        for addr in participants.iter() {
            self.send().direct_egld(&addr, &per_user);
        }
    }

    #[endpoint(extractWinners)]
    fn extract_winners(&self, num_winners: u64) -> ManagedVec<ManagedAddress> {
        let mut rng = RandomnessSource::default();
        let mut participants = self.participants();
        let mut winners = ManagedVec::new();
        for _ in 0..num_winners {
            let winner_index = rng.next_usize_in_range(0, participants.len());
            let winner = participants.get(winner_index);
            winners.push(winner.clone());
            participants.swap_remove(winner_index);
        }
        winners
    }

    #[view(getParticipants)]
    #[storage_mapper("participants")]
    fn participants(&self) -> VecMapper<ManagedAddress>;
}
