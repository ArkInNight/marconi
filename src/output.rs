pub struct Output {
    pub verbose: bool,
}

impl Output {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn banner() {
        println!("==================================");
        println!();
        println!("Marconi — Reverse:1999 Link Grabber");
        println!();
        println!("==================================");
        println!();
    }

    pub fn waiting_game() {
        println!("Waiting for Reverse:1999...");
        println!();
    }

    pub fn game_detected(pid: u32, name: &str) {
        println!("Game detected. (pid: {pid}, {name})");
        println!();
        println!("Waiting for summon history...");
        println!();
        println!("Now open the Summon Review page in-game (fresh, after this");
        println!("message), then import the link immediately — the token in");
        println!("the URL expires in about 5 minutes.");
        println!();
    }

    pub fn scanning(&self) {
        if self.verbose {
            println!("Scanning game memory...");
        }
    }

    pub fn link_found(url: &str) {
        println!("✔ Link Found");
        println!();
        println!("{url}");
        println!();
        println!("⚠ The token in this link expires in about 5 minutes.");
        println!("  Import it into your tracker right away. Do not reuse an");
        println!("  old link: if the tracker answers {{\"msg\":\"没有权限\",\"code\":-6}}");
        println!("  (\"no permission\"), the token has expired — exit the Summon");
        println!("  Review page / restart the game, reopen Summon Review, and");
        println!("  run marconi again to grab a fresh link.");
        println!();
    }

    pub fn copied() {
        println!("Copied to clipboard.");
        println!();
        println!("Done.");
        println!();
    }

    pub fn copy_failed() {
        println!("! Clipboard unavailable. URL above was not copied.");
        println!();
        println!("Done.");
        println!();
    }
}
