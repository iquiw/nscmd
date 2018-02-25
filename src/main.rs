extern crate tempdir;

use std::env;
use std::io::Result;
use std::os::unix::fs;
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

fn main() {
    match nscmd_main() {
        Ok(status) => match status.code() {
            Some(code) => exit(code),
            None => eprintln!("Process terminated by signal"),
        },
        Err(err) => eprintln!("{}", err),
    }
}

fn nscmd_main() -> Result<ExitStatus> {
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
            println!("{}", arg);
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
    fn new() -> Result<Self> {
        Ok(NsCmd {
            nscmd_dir: TempDir::new("nscmd-bin")?,
            nscmd_opts: process_args(),
        })
    }

    fn setup_trans(&self) -> Result<()> {
        for cmd_tr in &self.nscmd_opts.cmd_trans {
            let called = self.nscmd_dir.path().join(&cmd_tr.called_cmd);
            let actual = &cmd_tr.actual_cmd;
            println!("{} -> {}", actual, called.display());
            fs::symlink(actual, &called)?;
        }
        Ok(())
    }

    fn run_cmd(&self) -> Result<ExitStatus> {
        let mut cmd = Command::new(&self.nscmd_opts.cmd_args[0]);
        for arg in &self.nscmd_opts.cmd_args[1..] {
            cmd.arg(arg);
        }
        let mut path = self.nscmd_dir.path().as_os_str().to_os_string();
        if let Some(cur_path) = env::var_os("PATH") {
            path.push(":");
            path.push(&cur_path);
        }
        cmd.env("PATH", &path).status()
    }
}
