pub struct Output {
    pub verbose: bool,
}

impl Output {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn banner(&self) {
        println!("==================================");
        println!();
        println!("Marconi — Reverse:1999 Link Grabber");
        println!();
        println!("==================================");
        println!();
    }

    pub fn waiting_game(&self) {
        println!("Waiting for Reverse:1999...");
        println!();
    }

    pub fn game_detected(&self, pid: u32, name: &str) {
        println!("Game detected. (pid: {pid}, {name})");
        println!();
        println!("Waiting for summon history...");
        println!();
        println!("Open the Summon Review page in-game when ready.");
        println!();
    }

    pub fn scanning(&self) {
        if self.verbose {
            println!("Scanning game memory...");
        }
    }

    pub fn link_found(&self, url: &str) {
        println!("✔ Link Found");
        println!();
        println!("{url}");
        println!();
    }

    pub fn copied(&self) {
        println!("Copied to clipboard.");
        println!();
        println!("Done.");
        println!();
    }

    pub fn copy_failed(&self, url: &str) {
        println!("! Clipboard unavailable. URL above was not copied.");
        println!();
        println!("Done.");
        println!();
        let _ = url;
    }
}
