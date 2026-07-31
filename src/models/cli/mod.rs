use crate::{demo_scenarios, evaluate, Decision, Registries};

pub fn run_demo() {
    let registries = Registries::default();
    let scenarios = demo_scenarios();
    let mut stop = 0;
    let mut review = 0;
    let mut ready = 0;
    println!("SendSure deterministic Rust preflight demo");
    println!("Rules are deterministic Rust code; no LLM, blockchain API, database, or external risk service is used.\n");
    for (index, scenario) in scenarios.iter().enumerate() {
        let result = evaluate(&scenario.intent, &registries);
        match result.decision {
            Decision::Stop => stop += 1,
            Decision::Review => review += 1,
            Decision::Ready => ready += 1,
        }
        println!("{}. {} → {}", index + 1, scenario.name, result.decision);
        println!("   Rule: {}", result.triggered_rule_id);
        println!("   {}", result.explanation);
        println!("   Next: {}\n", result.recommended_next_step);
    }
    println!("Summary");
    println!("STOP: {stop}");
    println!("REVIEW: {review}");
    println!("READY: {ready}");
    println!("Total scenarios: {}", scenarios.len());
}
