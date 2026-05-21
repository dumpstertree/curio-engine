use std::time::Instant;

use curio_core::Ledger;
use curio_core::Nerve;
use curio_core::SystemComponent;
use curio_core::TabState;
use curio_core::built_in::record::sys_record_debug::SysRecordDebug;
use curio_core::built_in::record::sys_record_time::SysRecordTime;
use curio_core::built_in::stimulant::engine_commands::EngineCommands;

pub struct SystemComponentDefaultTime {
    instant: Instant,
    fps_average: Vec<f64>,
    timescale: f32,
    next_update: f64,
}

impl SystemComponentDefaultTime {
    pub fn new() -> Box<SystemComponentDefaultTime> {
        Box::new(SystemComponentDefaultTime {
            instant: Instant::now(),
            fps_average: Vec::new(),
            timescale: 1.0,
            next_update: 0.0,
        })
    }
}
// impl SystemComponentTime for SystemComponentDefaultTime {}
impl SystemComponent for SystemComponentDefaultTime {
    fn order(&self) -> i32 {
        1000
    }
    fn name(&self) -> String {
        "Time".to_owned()
    }
    fn init(&mut self, ledger: &mut Vec<Ledger>) {
        for ledger in ledger {
            ledger.write::<SysRecordTime>(|x| {
                x.target_frame_rate = 60.0;
            });
        }
    }

    fn tick(&mut self, ledger: &mut Vec<Ledger>, _: &mut Vec<Nerve>) {
        for ledger in ledger {
            let state_debug = ledger.read::<SysRecordDebug>();

            let cur_time = self.instant.elapsed().as_secs_f64();

            let fps = 1.0 / (cur_time - ledger.read::<SysRecordTime>().unscaled_time);
            self.fps_average.push(fps);

            while self.fps_average.len() > 5 {
                self.fps_average.remove(0);
            }

            let mut total = 0.0;
            for fps in &self.fps_average {
                total = total + fps;
            }
            let average_fps = (total / (self.fps_average.len() as f64)).round();

            // is paused
            let pause_timescale = self.timescale * if state_debug.is_paused { 0.0 } else { 1.0 };

            // edit the state
            ledger.write::<SysRecordTime>(|x| {
                x.frame_num = x.frame_num + 1;
                x.average_fps = average_fps as i32;
                // delta time
                x.unscaled_delta_time = (cur_time - x.unscaled_time) as f32;
                x.scaled_delta_time = x.unscaled_delta_time * pause_timescale;

                // time
                x.scaled_time = x.scaled_time + x.scaled_delta_time as f64;
                x.unscaled_time = cur_time;
            });
        }
    }

    fn refresh(&mut self, ledger: &mut Vec<Ledger>, _: &mut Vec<Nerve>) -> Vec<EngineCommands> {
        // get state
        let state_time = ledger[0].read::<SysRecordTime>();

        // calculate if we tick this frame
        let cur_time = self.instant.elapsed().as_secs_f64();
        let nxt_time = self.next_update;

        // if do tick send event
        let do_tick = cur_time >= nxt_time;
        if do_tick {
            // set next update
            self.next_update = (1.0 / state_time.target_frame_rate) as f64;

            // do tick
            return vec![EngineCommands::Redraw, EngineCommands::Tick];
        }
        // defualt
        return vec![EngineCommands::Redraw];
    }
}
