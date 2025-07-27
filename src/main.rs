// use recur_fn::{recur_fn, RecurFn};
use std::collections::HashMap;
use std::time::Instant;
use itertools::Itertools;
use itertools::concat;
use par_map::ParMap;



struct NumericalSemigroup {
    generators: Vec<i32>
}

impl NumericalSemigroup {
    fn dimension(&self) -> i32 { return self.generators.len() as i32; }
}

struct DynamicFactorizationsContext {
    nsg: NumericalSemigroup,
    stored_factorizations: HashMap<i32, Vec<Vec<i32>>>
}


impl DynamicFactorizationsContext {
    fn fd(&mut self, alpha: i32) -> Vec<Vec<i32>> { // Shorthand.
        self.factorizations_dynamic(alpha)
    }
    fn factorizations_dynamic(&mut self, alpha: i32) -> Vec<Vec<i32>> {
        if (alpha == 0) { return vec![vec![0;self.nsg.generators.len()]] } // zero is known to have exactly one factorization
        else if (alpha < 0) { return vec![] }   // negative elements have no factorizations
        else {
            let stored = self.stored_factorizations.get(&alpha);
            if (stored != None) {
                match stored {
                    Some(x) => { return x.clone(); }
                    None => return vec![]   // unreachable
                }
            }
            println!("Cache miss: {}", alpha);
            let predecessors = self.nsg.generators.iter().map(|x| alpha - x).collect::<Vec<i32>>();
            let predecessor_facs = predecessors.iter().map(|x| self.fd(*x));
            let mut result: Vec<Vec<i32>> = vec![];
            let _predecessor_facs_incremented = predecessor_facs.flat_map(
                |pred_facs| {
                    let cloned = pred_facs.clone();
                    cloned.iter().enumerate().map(
                        |(i,x)| increment_vector_along_axis(x.clone(), i)
                    ).collect::<Vec<Vec<i32>>>()}
            ).unique()
            .for_each(|x| result.push(x.clone()));
            self.stored_factorizations.insert(alpha, result.clone());
            return result;
        }
    }
    fn factorizations_dynamic_parallel(&mut self, alpha: i32) -> Vec<Vec<i32>> {
        //todo launch threads
                if (alpha == 0) { return vec![vec![0;self.nsg.generators.len()]] } // zero is known to have exactly one factorization
        else if (alpha < 0) { return vec![] }   // negative elements have no factorizations
        else {
            let stored = self.stored_factorizations.get(&alpha);
            if (stored != None) {
                match stored {
                    Some(x) => { return x.clone(); }
                    None => return vec![]   // unreachable
                }
            }
            println!("Cache miss: {}", alpha);
            let predecessors = self.nsg.generators.iter().map(|x| alpha - x).collect::<Vec<i32>>();
            let predecessor_facs = predecessors.iter().map(|x| self.fd(*x));
            let mut result: Vec<Vec<i32>> = vec![];
            let _predecessor_facs_incremented = predecessor_facs.par_flat_map(
                |pred_facs| {
                    let cloned = pred_facs.clone();
                    cloned.iter().enumerate().map(
                        |(i,x)| increment_vector_along_axis(x.clone(), i)
                    ).collect::<Vec<Vec<i32>>>()}
            ).unique()
            .for_each(|x| result.push(x.clone()));
            self.stored_factorizations.insert(alpha, result.clone());
            return result;
        }
    }
}

fn increment_vector_along_axis(x: Vec<i32>, i: usize) -> Vec<i32> {
    return x.iter().enumerate().map(|(j,y)| if (j == i) { *y+1 } else { *y }).collect();
}

fn main() {
    let nsg = NumericalSemigroup{generators:vec![13,33,37]};
    let mut context = DynamicFactorizationsContext{nsg:nsg, stored_factorizations: HashMap::new()};
    let time = Instant::now();
    let facs = context.factorizations_dynamic_parallel(1000);
    let facs_mapped: Vec<_> = facs.iter().map(|x| format!("{:?}", x)).collect();
    let facs_joined = facs_mapped.join(" ");
    println!("facs: {}", facs.len());//facs_joined);
    println!("time: {} ms", time.elapsed().as_millis());
}