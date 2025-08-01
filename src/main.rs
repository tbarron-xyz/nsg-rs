#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;

mod NumericalSemigroup;
use NumericalSemigroup::{DynamicFactorizationsContext, NumericalSemigroup as Nsg};

fn main() {
    let demoElement = 5000;
    let nsg = Nsg {
        generators: vec![13, 33, 37],
    };
    println!(
        "Generators: {}, {}, {}; element: {}",
        nsg.generators[0], nsg.generators[1], nsg.generators[2], demoElement
    );
    let mut context = DynamicFactorizationsContext {
        nsg: nsg.clone(),
        stored_factorizations: RwLock::new(HashMap::new()),
    };
    let time: Instant = Instant::now();
    let facs;
    facs = context.factorizations_dynamic_parallel(demoElement);
    println!("time parallel: {} ms", time.elapsed().as_millis());
    println!("facs: {}", facs.len());
    let mut context2 = DynamicFactorizationsContext {
        nsg: nsg.clone(),
        stored_factorizations: RwLock::new(HashMap::new()),
    };
    let time2: Instant = Instant::now();
    let facs2: Vec<Vec<i32>> = context2.factorizations_dynamic_singleThread(demoElement);
    println!("time serial: {} ms", time2.elapsed().as_millis());
    println!("facs: {}", facs2.len());
}
