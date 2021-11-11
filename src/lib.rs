mod sauce;
mod common;


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::Path;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    #[ignore]
    fn find_sauce() {
        let path = Path::new("test.jpg");
        /*let file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", path.display(), why),
            Ok(file) => file,
        };
        */
        let res = sauce::sauce_finder::find_sauce(&path);
        if let Err(err) = res {
            println!("error:\n{}", err.to_string());
        }
    }
}
