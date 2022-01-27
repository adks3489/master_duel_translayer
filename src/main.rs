use master_duel_translayer::{attach_process};

fn main() {
    let p = attach_process().unwrap();
    println!("pid: {}", p.pid);
}
