use crate::context::Context;

pub fn print_values(ctx: Context) {
    for ty in ctx.types() {
        println!("-------- {}:", ty);
        for val in ctx.values_of(&ty) {
            println!("{}", val);
        }
    }
}

pub fn permute<T: Clone>(vals: Vec<Vec<T>>) -> Vec<Vec<T>> {
    fn permute_rec<T: Clone>(vals: &Vec<Vec<T>>, perms: &mut Vec<Vec<T>>, curr: Vec<T>) {
        if curr.len() == vals.len() {
            perms.push(curr);
            return;
        }
        for val in vals[curr.len()].iter() {
            let mut curr = curr.clone();
            curr.push(val.clone());
            permute_rec(vals, perms, curr);
        }
    }
    let mut perms = vec![];
    permute_rec(&vals, &mut perms, vec![]);
    perms
}
