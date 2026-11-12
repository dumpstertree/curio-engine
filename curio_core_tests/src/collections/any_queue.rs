#[cfg(test)]
mod tests {
    use curio_core::AnyQueue;

    #[derive(Clone)]
    struct QueueProbeA {
        pub value: i32,
    }
    #[derive(Clone)]
    struct QueueProbeB {
        pub value: i32,
    }

    // =========================================================
    // Push / drain
    // =========================================================

    #[test]
    fn push_and_drain_returns_owned_values_in_order() {
        let queue = AnyQueue::new();
        queue.push(QueueProbeA { value: 1 });
        queue.push(QueueProbeA { value: 2 });

        let drained = queue.drain<QueueProbeA>();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].value, 1);
        assert_eq!(drained[1].value, 2, "drain preserves insertion order");

        assert_eq!(queue.drain<QueueProbeA>().len(), 0, "drain empties matching items");
    }

    #[test]
    fn drain_only_removes_matching_type() {
        let queue = AnyQueue::new();
        queue.push(QueueProbeA { value: 1 });
        queue.push(QueueProbeB { value: 2 });
        queue.push(QueueProbeA { value: 3 });

        let drained = queue.drain<QueueProbeA>();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].value, 1);
        assert_eq!(drained[1].value, 3);

        assert_eq!(queue.len(), 1, "the non-matching probe must stay in the queue");
        assert!(queue.has_type<QueueProbeB>());
    }

    #[test]
    fn push_boxed_roundtrip() {
        let queue = AnyQueue::new();
        queue.push_boxed(Box::new(QueueProbeA { value: 4 }));

        let drained = queue.drain<QueueProbeA>();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].value, 4);
    }

    // =========================================================
    // Views
    // =========================================================

    #[test]
    fn get_all_is_a_view_without_draining() {
        let queue = AnyQueue::new();
        queue.push(QueueProbeA { value: 1 });

        let seen = queue.get_all<QueueProbeA>();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].value, 1);

        assert_eq!(queue.get_all<QueueProbeB>().len(), 0);
        assert_eq!(queue.len(), 1, "get_all must not consume");
    }

    #[test]
    fn has_type_reports_membership() {
        let queue = AnyQueue::new();
        queue.push(QueueProbeB { value: 7 });

        assert!(queue.has_type<QueueProbeB>());
        assert!(
            !queue.has_type<QueueProbeA>(),
            "absent types must report false",
        );
    }

    #[test]
    fn length_counts_all_types() {
        let queue = AnyQueue::new();
        assert!(queue.is_empty());

        queue.push(QueueProbeA { value: 1 });
        queue.push(QueueProbeB { value: 2 });
        queue.push(QueueProbeA { value: 3 });
        assert_eq!(queue.len(), 3);

        queue.drain<QueueProbeA>();
        assert_eq!(queue.len(), 1);

        queue.drain<QueueProbeB>();
        assert!(queue.is_empty());
    }

    // =========================================================
    // Interleaving
    // =========================================================

    #[test]
    fn partial_drain_keeps_the_rest_and_order_survives() {
        let queue = AnyQueue::new();
        queue.push(QueueProbeA { value: 1 });
        queue.push(QueueProbeB { value: 2 });
        queue.push(QueueProbeA { value: 3 });

        queue.drain<QueueProbeA>();
        queue.push(QueueProbeA { value: 4 });

        let second = queue.drain<QueueProbeA>();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].value, 4, "only the post-drain push remains");
    }
}
