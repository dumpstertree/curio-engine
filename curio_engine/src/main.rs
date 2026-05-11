use crate::curio_cabinet::CurioCabinet;

pub mod adapter_system_gpu;
pub mod curio_cabinet;
pub fn main() {
    CurioCabinet::put_on_display();
}
