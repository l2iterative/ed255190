use risc0_zkvm::guest::env;
use ed255190_guest::{Evaluator, DualTableEvaluator, ComputeHintBuffer, TEPoint};
use serde::{Serialize, Deserialize};
use l2r0_small_serde::from_slice_compact;
use l2r0_profiler_guest::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub s1: [u32; 8],
    pub s2: [u32; 8],
    pub g2_x: [u32; 8],
    pub g2_y: [u32; 8],
}

fn main() {
    l2r0_profiler_guest::init_trace_logger();
    start_timer!("Total");

    start_timer!("Read the task slice");
    let task_slice: Vec<u32> = env::read();
    stop_start_timer!("Convert the task slice to task");
    let task: Task = from_slice_compact(&task_slice).unwrap();
    stop_start_timer!("Get the compute hint length");
    let compute_hint_length: u32 = env::read();
    stop_start_timer!("Initialize the compute hint");

    stop_start_timer!("Evaluation");
    let mut compute_hint_provider = ComputeHintBuffer::new(compute_hint_length as usize);
    let eval1 = Evaluator::new(
        &task.s1,
        &task.s2,
        &TEPoint {
            x: task.g2_x,
            y: task.g2_y
        },
    );
    let eval2 = DualTableEvaluator::new(
        &task.s1,
        &task.s2,
    );

    let res1 = eval1.evaluate(&mut compute_hint_provider);
    let res2 = eval2.evaluate(&mut compute_hint_provider);
    let res3 = eval1.evaluate(&mut compute_hint_provider);
    let res4 = eval2.evaluate(&mut compute_hint_provider);
    let res5 = eval1.evaluate(&mut compute_hint_provider);
    let res6 = eval2.evaluate(&mut compute_hint_provider);
    let res7 = eval1.evaluate(&mut compute_hint_provider);
    let res8 = eval2.evaluate(&mut compute_hint_provider);
    assert!(matches!(res1, Ok(_)));
    assert!(matches!(res2, Ok(_)));
    assert!(matches!(res3, Ok(_)));
    assert!(matches!(res4, Ok(_)));
    assert!(matches!(res5, Ok(_)));
    assert!(matches!(res6, Ok(_)));
    assert!(matches!(res7, Ok(_)));
    assert!(matches!(res8, Ok(_)));

    let compute_result_1 = match res1 {
        Ok(v) => v,
        Err(_) => {
            unreachable!()
        }
    };
    let compute_result_2 = match res2 {
        Ok(v) => v,
        Err(_) => {
            unreachable!()
        }
    };
    stop_timer!();

    env::commit_slice(&compute_result_1);
    env::commit_slice(&compute_result_2);
    stop_timer!();
}