extern crate proc_macro;

use proc_macro::{ TokenStream };
use quote::{ quote };
use std::path::Path;
use proc_macro_hack::proc_macro_hack;

#[proc_macro]
pub fn declare(input: TokenStream) -> TokenStream {
    let identifier = proc_macro2::TokenStream::from(input);
    let cwd = std::env::current_dir().unwrap();
    let head = cwd.join(".git/HEAD").to_str().unwrap().to_string();
    let mut interesting_files = vec![head];
    let refs = Path::new(".git/refs/heads");
    for entry in refs.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            interesting_files.push(cwd.join(entry.path()).to_str().unwrap().to_string());
        }
    }
    let vec = std::process::Command::new("git")
            .args(&["describe", "--always", "--dirty"])
            .output()
        .expect("failed to execute git").stdout;
    let name = std::str::from_utf8(&vec[..vec.len()-1]).expect("non-utf8 error?!");
    let x = quote!{
        fn __unused_by_git_version() {
            // This is included simply to cause cargo to rebuild when
            // a new commit is made.
            #( include_str!(#interesting_files); )*
        }
        const #identifier: &'static str = {
            #name
        };
    };
    // println!("tokens are {}", x);
    x.into()
}

/// Use the given template to create a string.
///
/// You can think of this as being kind of like `format!` on strange drugs.
#[proc_macro_hack]
pub fn git_describe(input: TokenStream) -> TokenStream {
    let mut args = Vec::new();
    let mut next_arg = String::new();
    for t in input.into_iter() {
        let x = t.to_string();
        let last_char = next_arg.clone().pop(); // yes, this is terribly wasteful...
        if next_arg.len() > 0 && next_arg != "-" && next_arg != "--" && x == "-"
            && last_char != Some('=')
        {
            args.push(next_arg.clone());
            next_arg = String::new();
        }
        next_arg.extend(x.chars());
    }
    if next_arg.len() > 0 {
        args.push(next_arg);
    }
    let cwd = std::env::current_dir().unwrap();
    let head = cwd.join(".git/HEAD").to_str().unwrap().to_string();
    let mut interesting_files = vec![head];
    let refs = Path::new(".git/refs/heads");
    for entry in refs.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            interesting_files.push(cwd.join(entry.path()).to_str().unwrap().to_string());
        }
    }
    let mut cmd = std::process::Command::new("git");
    cmd.args(&["describe", "--always"]);
    for a in args.iter() {
        cmd.arg(&a);
    }
    let vec = cmd.output().expect("failed to execute git").stdout;
    if vec.len() == 0 {
        panic!("the command {:?} exited without returning a description", cmd);
    }
    let name = std::str::from_utf8(&vec[..vec.len()-1]).expect("non-utf8 error?!");
    let x = quote!{
        {
            // This is included simply to cause cargo to rebuild when
            // a new commit is made.
            #( include_str!(#interesting_files); )*
            #name
        }
    };
    x.into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
