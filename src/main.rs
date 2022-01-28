use master_duel_translayer::{attach_process, capture};

fn main() {
    let p = attach_process().unwrap();
    println!("pid: {}", p.pid);
    capture(&p);
}
