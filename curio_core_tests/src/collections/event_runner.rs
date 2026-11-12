#[cfg(test)]
mod tests {
    use curio_core::EventRunner;

    #[derive(Clone)]
    struct RunnerEvent {
        pub value: i32,
    }
    #[derive(Clone)]
    struct RunnerData {
        pub log: Vec<i32>,
    }

    // Receivers mirror how the engine uses EventRunner: plain fn-pointers
    // (see optional_systems/cards/src/card_event_runner.rs) that see every
    // event and may emit follow-up events.
    fn reciever_log(event: &RunnerEvent, data: &mut RunnerData) -> Vec<RunnerEvent> {
        data.log.push(event.value);
        vec![]
    }

    fn reciever_chain(event: &RunnerEvent, data: &mut RunnerData) -> Vec<RunnerEvent> {
        data.log.push(event.value);
        if event.value > 0 {
            return vec![RunnerEvent { value: event.value - 1 }];
        }
        vec![]
    }

    fn mark_first(event: &RunnerEvent, data: &mut RunnerData) -> Vec<RunnerEvent> {
        data.log.push(event.value * 100);
        vec![]
    }

    fn mark_second(event: &RunnerEvent, data: &mut RunnerData) -> Vec<RunnerEvent> {
        data.log.push(event.value * 100 + 1);
        vec![]
    }

    // =========================================================
    // Enqueue / drain
    // =========================================================

    #[test]
    fn receiver_sees_every_enqueued_event_in_order() {
        let mut runner = EventRunner::new(vec![reciever_log]);
        let mut data = RunnerData { log: vec![] };

        runner.enqueue(&RunnerEvent { value: 1 });
        runner.enqueue(&RunnerEvent { value: 2 });
        runner.post_and_drain(&data);

        assert_eq!(data.log, vec![1, 2]);
    }

    #[test]
    fn empty_queue_is_a_no_op() {
        let mut runner = EventRunner::new(vec![reciever_log]);
        let mut data = RunnerData { log: vec![] };

        runner.post_and_drain(&data);
        assert_eq!(data.log.len(), 0);
    }

    #[test]
    fn enqueue_clones_the_event() {
        let mut runner = EventRunner::new(vec![reciever_log]);
        let mut data = RunnerData { log: vec![] };

        let mut original = RunnerEvent { value: 5 };
        runner.enqueue(&original);
        original.value = 99;

        runner.post_and_drain(&data);
        assert_eq!(data.log, vec![5], "the queued copy must be independent of the original");
    }

    // =========================================================
    // Generated events
    // =========================================================

    #[test]
    fn generated_events_are_processed_fifo_until_exhausted() {
        let mut runner = EventRunner::new(vec![reciever_chain]);
        let mut data = RunnerData { log: vec![] };

        runner.enqueue(&RunnerEvent { value: 3 });
        runner.post_and_drain(&data);

        assert_eq!(data.log, vec![3, 2, 1, 0], "children are appended to the queue tail and each is visited once");
    }

    // =========================================================
    // Fan-out
    // =========================================================

    #[test]
    fn every_receiver_gets_every_event_in_registration_order() {
        let mut runner = EventRunner::new(vec![mark_first, mark_second]);
        let mut data = RunnerData { log: vec![] };

        runner.enqueue(&RunnerEvent { value: 7 });
        runner.enqueue(&RunnerEvent { value: 9 });
        runner.post_and_drain(&data);

        assert_eq!(
            data.log,
            vec![700, 701, 900, 901],
            "per event: receivers run in registration order, events run in enqueue order",
        );
    }

    // Note: an unbounded receiver (one that always returns a new event) makes
    // post_and_drain loop forever - there is no cycle guard. Not testable
    // without hanging the test binary, so it is only documented here.
}
