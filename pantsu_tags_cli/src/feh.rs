use std::io;
use std::process::{Child, Command, Stdio};

pub struct FehProcesses {
    processes: Vec<Child>,
}

impl FehProcesses {
    pub fn new_empty() -> FehProcesses {
        FehProcesses {
            processes: Vec::new(),
        }
    }
    pub fn kill(mut self) {
        for proc in &mut self.processes {
            let _ = proc.kill(); // don't care if kill fails
        }
    }
}

pub fn feh_available() -> bool {
    which::which("sh").is_ok() &&
        which::which("feh").is_ok()
}

//feh --info 'echo "$((%u -1))"' https://img3.gelbooru.com/images/bb/62/bb626c2a621cbc1642256c0ebefbd219.jpg https://img3.gelbooru.com/images/12/ee/12ee1ac61779f5ccfcc383485c7c3191.png

// if no FehProcesses is available, create an new one with FehProcesses::new_empty()
pub fn feh_display_images<'a, I>(images: I, label: &str, mut feh_procs: FehProcesses) -> FehProcesses
where I: Iterator<Item = &'a str>
{
    let cmd = make_feh_command(images, label);
    match spawn_process(&cmd) {
        Ok(proc) => {
            feh_procs.processes.push(proc);
            feh_procs
        },
        Err(_) => {
            feh_procs.kill();
            FehProcesses::new_empty()
        },
    }
}

fn make_feh_command<'a, I>(images: I, label: &str) -> String
where I: Iterator<Item = &'a str>
{
    let mut cmd = format!("feh -q.ZB black --info \'echo \"%u/%l {}\"\'", label);
    for image in images {
        cmd.push_str(" \'");
        cmd.push_str(image);
        cmd.push('\'');
    }
    cmd
}

fn spawn_process(command: &str) -> io::Result<Child>{
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}