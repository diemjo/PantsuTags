use std::io;
use std::io::BufReader;
use std::process::{Child, ChildStdout, Command, Stdio};

pub struct FehProcesses {
    processes: Vec<Child>,
    reader: Option<BufReader<ChildStdout>>,
}

impl FehProcesses {
    pub fn kill(mut self) {
        for proc in &mut self.processes {
            let _ = proc.kill(); // don't care if kill fails
        }
    }

    pub fn take_reader(&mut self) -> Option<BufReader<ChildStdout>> {
        self.reader.take()
    }

    pub fn give_reader(&mut self, reader: BufReader<ChildStdout>) {
        self.reader = Some(reader);
    }

    fn new_empty() -> Self {
        FehProcesses {
            processes: Vec::new(),
            reader: None,
        }
    }

    fn new(processes: Vec<Child>, child_out: Option<ChildStdout>) -> FehProcesses {
        let buf_reader = match child_out {
            Some(stdout) => Some(BufReader::new(stdout)),
            None =>  None,
        };
        FehProcesses {
            processes,
            reader: buf_reader,
        }
    }
}

pub fn feh_available() -> bool {
    which::which("sh").is_ok() &&
        which::which("feh").is_ok()
}


//feh --info 'echo "$((%u -1))"' https://img3.gelbooru.com/images/bb/62/bb626c2a621cbc1642256c0ebefbd219.jpg https://img3.gelbooru.com/images/12/ee/12ee1ac61779f5ccfcc383485c7c3191.png

pub fn feh_compare_image(image: &str, other_images: &Vec<&str>, image_label: &str, other_images_label: &str) -> FehProcesses {
    let command_image = format!("feh -q.ZB black --info \'echo \"{}\"\' \'{}\'", image_label, image);

    let mut command_other_images = format!("feh -q.ZB black --action \"echo s\" --info \'echo \"$((%u -1)) {}\"\'", other_images_label);
    for &image in other_images {
        command_other_images.push_str(" \'");
        command_other_images.push_str(image);
        command_other_images.push('\'');
    }

    let mut cmd_image = match spawn_process(&command_image, Stdio::piped()) {
        Ok(cmd) => cmd,
        Err(_) => return FehProcesses::new_empty(),
    };

    let mut cmd_other_images = match spawn_process(&command_other_images, Stdio::piped()) {
        Ok(cmd) => cmd,
        Err(_) => {
            let _ = cmd_image.kill();
            return FehProcesses::new_empty();
        }
    };
    let child_out = cmd_other_images.stdout.take();
    FehProcesses::new(vec![cmd_image, cmd_other_images], child_out)
}

fn spawn_process(command: &str, stdout_mode: Stdio) -> io::Result<Child>{
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(stdout_mode)
        .stderr(Stdio::null())
        .spawn()
}