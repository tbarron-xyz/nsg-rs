#![allow(non_snake_case)]

use futures::future::join_all;
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;

#[derive(Clone)]
struct NumericalSemigroup {
    generators: Vec<i32>,
}

impl NumericalSemigroup {
    fn dimension(&self) -> usize {
        return self.generators.len();
    }
}

struct DynamicFactorizationsContext {
    nsg: NumericalSemigroup,
    stored_factorizations: RwLock<HashMap<i32, Vec<Vec<i32>>>>,
}

fn isAllZeroesLeftOfIndex(vec: &Vec<i32>, indexArg: usize) -> bool {
    return vec
        .iter()
        .enumerate()
        .all(|(index, number)| index >= indexArg || *number == 0);
}

fn facs_dynamic_noMut<'b>(
    cache: &'b HashMap<i32, Vec<Vec<i32>>>,
    alpha: i32,
    dim: usize,
) -> Vec<Vec<i32>> {
    // contract: only call if you know it's in the cache already.
    if (alpha == 0) {
        return vec![vec![0; dim]];
    }
    // zero is known to have exactly one factorization
    else if (alpha < 0) {
        return vec![];
    }
    // negative elements have no factorizations
    else {
        // println!("Getting cached facs for {}", alpha);
        let stored = cache.get(&alpha);
        if (stored != None) {
            // println!("Got cached facs for {}", alpha);
            match stored {
                Some(x) => {
                    return x.clone();
                }
                None => return vec![], // unreachable
            }
        }
        return vec![]; // per contract, will not be reached
    }
}

impl DynamicFactorizationsContext {
    fn factorizations_best(&mut self, alpha: i32) -> Vec<Vec<i32>> {
        // the fastest/best one.
        return self.factorizations_dynamic_singleThread(alpha);
    }
    fn elementIsMember(&mut self, alpha: i32) -> bool {
        return !self.factorizations_best(alpha).is_empty();
    }

    fn fd(&mut self, alpha: i32) -> Vec<Vec<i32>> {
        // Shorthand.
        self.factorizations_dynamic_singleThread(alpha)
    }
    fn factorizations_dynamic_singleThread(&mut self, alpha: i32) -> Vec<Vec<i32>> {
        if (alpha == 0) {
            return vec![vec![0; self.nsg.generators.len()]];
        }
        // zero is known to have exactly one factorization
        else if (alpha < 0) {
            return vec![];
        }
        // negative elements have no factorizations
        else {
            {
                let cache1 = self.stored_factorizations.read().unwrap();
                let stored = cache1.get(&alpha);
                if (stored != None) {
                    match stored {
                        Some(x) => {
                            return x.clone();
                        }
                        None => return vec![], // unreachable
                    }
                }
            }
            // println!("Cache miss: {}", alpha);
            let predecessors = self
                .nsg
                .generators
                .iter()
                .map(|x| alpha - x)
                .collect::<Vec<i32>>();
            let predecessor_facs = predecessors.iter().map(|x| self.fd(*x));
            let mut result: Vec<Vec<i32>> = vec![];
            let _predecessor_facs_incremented = predecessor_facs
                .enumerate()
                .flat_map(|(predecessorIndex, pred_facs)| {
                    pred_facs
                        .iter()
                        .filter(|x| isAllZeroesLeftOfIndex(x, predecessorIndex))
                        .map(|x| increment_vector_along_axis(x.clone(), predecessorIndex))
                        .collect::<Vec<Vec<i32>>>()
                })
                // .unique()
                .for_each(|x| result.push(x));
            self.stored_factorizations
                .write()
                .unwrap()
                .insert(alpha, result.clone());
            return result;
        }
    }

    fn facs_dynamic_noMut<'b>(
        cache: &'b HashMap<i32, Vec<Vec<i32>>>,
        alpha: i32,
        dim: usize,
    ) -> Vec<Vec<i32>> {
        // contract: only call if you know it's in the cache already.
        if (alpha == 0) {
            return vec![vec![0; dim]];
        }
        // zero is known to have exactly one factorization
        else if (alpha < 0) {
            return vec![];
        }
        // negative elements have no factorizations
        else {
            // println!("Getting cached facs for {}", alpha);
            let stored = cache.get(&alpha);
            if (stored != None) {
                // println!("Got cached facs for {}", alpha);
                match stored {
                    Some(x) => {
                        return x.clone();
                    }
                    None => return vec![], // unreachable
                }
            }
            return vec![]; // per contract, will not be reached
        }
    }

    fn get_facs_dynamic_oneElement(
        threadId: i32,
        gens: &Vec<i32>,
        cache: &RwLock<HashMap<i32, Vec<Vec<i32>>>>,
        baseElement: i32,
        dim: usize,
    ) -> (i32, Vec<Vec<i32>>) {
        // one worker
        let tid = threadId.clone();
        let gens3 = gens.clone();

        // handles.push(tokio::spawn(async move {
        let roundThreadElement = baseElement + tid;
        // println!("element: {}", roundThreadElement);
        let predecessors = gens3.iter().map(|x| roundThreadElement - *x).collect_vec();

        let predecessor_facs = predecessors
            .iter()
            .map(|p| {
                DynamicFactorizationsContext::facs_dynamic_noMut(&(cache.read().unwrap()), *p, dim)
            })
            .collect_vec();
        // println!("pfs: {}", predecessor_facs.iter().flatten().collect_vec().len());

        let mut result: Vec<Vec<i32>> = vec![];
        // println!("Incrementing facs");
        let _predecessor_facs_incremented = predecessor_facs
            .iter()
            .enumerate()
            .flat_map(|(predecessorIndex, pred_facs)| {
                pred_facs
                    .iter()
                    .filter(|x| isAllZeroesLeftOfIndex(x, predecessorIndex))
                    .map(|x| {
                        // println!("incrementing");
                        increment_vector_along_axis(x.clone(), predecessorIndex)
                    })
                    .collect_vec()
            })
            .for_each(|x| result.push(x));
        // println!("returning {} facs", result.len());
        // println);
        return (roundThreadElement, result);
    }

    async fn factorizations_dynamic_parallel(&mut self, alpha: i32) -> Vec<Vec<i32>> {
        if (alpha == 0) {
            return vec![vec![0; self.nsg.generators.len()]];
        }
        // zero is known to have exactly one factorization
        else if (alpha < 0) {
            return vec![];
        }
        // negative elements have no factorizations
        else {
            {
                let cache1 = self.stored_factorizations.read().unwrap();
                let stored = cache1.get(&alpha);
                if (stored != None) {
                    match stored {
                        Some(x) => {
                            return x.clone();
                        }
                        None => return vec![], // unreachable
                    }
                }
            }
            let cache = &self.stored_factorizations;
            let smallestGen = self.nsg.generators[0];
            let gens1 = self.nsg.generators.clone();
            let dim = self.nsg.dimension();
            // let gens = &gens1;

            // let mut res = vec![];
            for round in ((0..=alpha).step_by(self.nsg.generators[0] as usize)) {
                // let rounds = ((0..=alpha).step_by(self.nsg.generators[0] as usize)).map(move |baseElement| /* async move  */ {
                // println!("baseElement: {}", baseElement);
                // one round of each worker
                let baseElement = round;
                let be2 = baseElement.clone();
                let gens2 = gens1.clone();

                let mut outputs = vec![];

                std::thread::scope(|s| {
                    let mut handles = vec![];
                    for threadId in (0..smallestGen) {
                        // println!("threadId: {}", threadId);
                        // one worker
                        let gens3 = gens2.clone();

                        let proc = move || {
                            return DynamicFactorizationsContext::get_facs_dynamic_oneElement(
                                threadId,
                                &gens3,
                                cache,
                                baseElement,
                                dim,
                            );
                        };
                        handles.push(s.spawn(proc));
                    }
                    for h in handles {
                        outputs.push(h.join().unwrap());
                    }
                });
                outputs
                    .iter()
                    .enumerate()
                    .map(|(index, (element, result))| {
                        // for v in result {
                        //     println!("{:?},{:?},{:?}",v[0], v[1], v[2]);
                        // }
                        // println!("storing {} facs for {}", result.len(), *element);
                        let mut db = cache.write().unwrap();
                        // println!("got db ");

                        db.insert(*element, result.clone());
                        // println!("Wrote");
                        // let getResult = db.get(element).unwrap();
                        // println!("Got value back: {:?}", *getResult);
                    })
                    .collect_vec();
            }

            return self.stored_factorizations.read().unwrap()[&alpha].clone();
        };
    }
}

fn increment_vector_along_axis(x: Vec<i32>, i: usize) -> Vec<i32> {
    return x
        .iter()
        .enumerate()
        .map(|(j, y)| if (j == i) { *y + 1 } else { *y })
        .collect();
}

#[tokio::main(worker_threads = 8)]
async fn main() {
    let demoElement = 5000;
    let nsg = NumericalSemigroup {
        generators: vec![13, 33, 37],
    };
    let mut context = DynamicFactorizationsContext {
        nsg: nsg.clone(),
        stored_factorizations: RwLock::new(HashMap::new()),
    };
    let time: Instant = Instant::now();
    let facs;
    {
        // let facs: Vec<Vec<i32>>
        facs = context.factorizations_dynamic_parallel(demoElement).await;
    }
    let facs_mapped: Vec<_> = facs.iter().map(|x| format!("{:?}", x)).collect();
    let facs_joined = facs_mapped.join(" ");
    println!("time parallel: {} ms", time.elapsed().as_millis());
    println!("facs: {}", facs.len()); //facs_joined);
    let mut context2 = DynamicFactorizationsContext {
        nsg: nsg.clone(),
        stored_factorizations: RwLock::new(HashMap::new()),
    };
    let time2: Instant = Instant::now();
    let facs2: Vec<Vec<i32>> = context2.factorizations_dynamic_singleThread(demoElement);
    println!("time serial: {} ms", time2.elapsed().as_millis());
    println!("facs: {}", facs2.len()); //facs_joined);
}
