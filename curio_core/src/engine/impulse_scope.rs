#[derive(Clone, PartialEq)]
/// The Curios on the network in which an Impulse will be transmitted
pub enum ImpulseScope {
    All,
    Instance,
    ConnectedHost,
    ConnectedPeers,
}
