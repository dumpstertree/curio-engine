pub trait IState<T>
where
    T: Clone,
{
    fn default() -> T;
    fn id() -> i32;
}
