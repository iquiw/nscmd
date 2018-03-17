extern crate tempdir;

use std::env;
use std::error::Error;
use std::fmt::Display;
use std::os::unix::fs;
use std::path::Path;
use std::process::{exit, Command, ExitStatus};

use tempdir::TempDir;

struct CmdTrans {
    called_cmd: String,
    actual_cmd: String,
}

struct NsCmdArgs {
    cmd_trans: Vec<CmdTrans>,
    cmd_args: Vec<String>,
}

struct NsCmd {
    nscmd_dir: TempDir,
    nscmd_opts: NsCmdArgs,
}

#[derive(Debug)]
struct NsCmdErr(String);

impl Display for NsCmdErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str(&self.0)
    }
}

impl Error for NsCmdErr {
    fn description(&self) -> &str {
        &self.0
    }
}

fn main() {
    match nscmd_main() {
        Ok(status) => match status.code() {
            Some(code) => exit(code),
            None => eprintln!("Process terminated by signal"),
        },
        Err(err) => eprintln!("{}", err),
    }
}

fn nscmd_main() -> Result<ExitStatus, Box<Error>> {
    let nscmd = NsCmd::new()?;
    nscmd.setup_trans()?;
    nscmd.run_cmd()
}

fn process_args() -> NsCmdArgs {
    let mut trans = Vec::new();
    let mut args = Vec::new();
    let mut trans_end = false;

    for arg in env::args().skip(1) {
        if trans_end {
            args.push(arg.to_string());
        } else if let Some(index) = arg.find("=") {
            trans.push(CmdTrans {
                called_cmd: arg[0..index].to_string(),
                actual_cmd: arg[index + 1..].to_string(),
            });
        } else {
            trans_end = true;
            args.push(arg.to_string());
        }
    }
    NsCmdArgs {
        cmd_trans: trans,
        cmd_args: args,
    }
}

impl NsCmd {
    fn new() -> Result<Self, Box<Error>> {
        Ok(NsCmd {
            nscmd_dir: TempDir::new("nscmd-bin")?,
            nscmd_opts: process_args(),
        })
    }

    fn setup_trans(&self) -> Result<(), Box<Error>> {
        for cmd_tr in &self.nscmd_opts.cmd_trans {
            let called = self.nscmd_dir.path().join(&cmd_tr.called_cmd);
            let actual = Path::new(&cmd_tr.actual_cmd);
            if actual.is_file() {
                println!("{} -> {}", &cmd_tr.actual_cmd, called.display());
            } else {
                return Err(Box::new(NsCmdErr(format!("{} does not exist", &cmd_tr.actual_cmd))));
            }
            fs::symlink(actual, &called)?;
        }
        Ok(())
    }

    fn run_cmd(&self) -> Result<ExitStatus, Box<Error>> {
        let mut cmd = Command::new(&self.nscmd_opts.cmd_args[0]);
        for arg in &self.nscmd_opts.cmd_args[1..] {
            cmd.arg(arg);
        }
        let mut path = self.nscmd_dir.path().as_os_str().to_os_string();
        if let Some(cur_path) = env::var_os("PATH") {
            path.push(":");
            path.push(&cur_path);
        }
        Ok(cmd.env("PATH", &path).status()?)
    }
}
