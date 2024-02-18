use std::io::Read;

fn main() {
    // コマンドライン引数からファイル名を取得して、ファイルを開き、その中身を表示する
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("ファイル名を指定してください");
        std::process::exit(1);
    }
    let filename = &args[1];
    let mut file = match std::fs::File::open(filename) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("ファイルを開けませんでした: {}", err);
            std::process::exit(1);
        }
    };
    let mut contents = String::new();
    if let Err(err) = file.read_to_string(&mut contents) {
        eprintln!("ファイルを読み込めませんでした: {}", err);
        std::process::exit(1);
    }
    println!("{}", contents);
}
