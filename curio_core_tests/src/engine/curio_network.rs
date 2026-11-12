#[cfg(test)]
mod tests {
    use curio_core::{CurioNetwork, CurioNetworkParticipant, NetworkModes};

    // =========================================================
    // Participants
    // =========================================================

    #[test]
    fn all_and_me_report_the_same_participants() {
        let seats = vec![
            CurioNetworkParticipant::new(111, NetworkModes::LocalHost),
            CurioNetworkParticipant::new(222, NetworkModes::LocalPeer),
        ];
        let network = CurioNetwork::new(seats.clone(), 1);

        assert_eq!(network.all().len(), 2);
        assert_eq!(network.all()[0].guid, 111);

        assert_eq!(network.me().guid, 222);
        assert_eq!(network.me().mode, NetworkModes::LocalPeer);
    }

    #[test]
    fn all_returns_a_read_only_view() {
        let seats = vec![CurioNetworkParticipant::new(111, NetworkModes::LocalHost)];
        let network = CurioNetwork::new(seats.clone(), 0);

        assert_eq!(network.all(), &seats[0..]);
    }

    // =========================================================
    // Bad indices
    // =========================================================

    #[test]
    #[should_panic]
    fn empty_network_has_no_me() {
        // BUG surface: Ledger::new_custom builds exactly this empty network,
        // then anything calling me() indexes out of bounds
        let network = CurioNetwork::new(Vec::new(), 0);
        network.me();
    }

    #[test]
    #[should_panic]
    fn me_index_out_of_bounds_panics() {
        let seats = vec![CurioNetworkParticipant::new(111, NetworkModes::LocalHost)];
        let network = CurioNetwork::new(seats, 5);
        network.me();
    }
}
