use std::path::PathBuf;
use std::sync::Mutex;

pub type RestoreHook = Box<dyn FnOnce() + Send>;

pub struct CleanupManager {
    dirs: Mutex<Vec<PathBuf>>,
    hooks: Mutex<Vec<RestoreHook>>,
    running: Mutex<bool>,
}

impl CleanupManager {
    pub fn new(dirs: Vec<PathBuf>) -> Self {
        Self {
            dirs: Mutex::new(dirs),
            hooks: Mutex::new(Vec::new()),
            running: Mutex::new(false),
        }
    }

    #[allow(dead_code)]
    pub fn register(&self, dirs: Vec<PathBuf>) {
        if let Ok(mut d) = self.dirs.lock() {
            d.extend(dirs);
        }
    }

    #[allow(dead_code)]
    pub fn add_hook(&self, hook: RestoreHook) {
        if let Ok(mut h) = self.hooks.lock() {
            h.push(hook);
        }
    }

    pub fn cleanup(&self) {
        let mut running = match self.running.lock() {
            Ok(r) => r,
            Err(poisoned) => poisoned.into_inner(),
        };
        if *running {
            return;
        }
        *running = true;

        let hooks = self.hooks.lock().map(|mut h| std::mem::take(&mut *h));
        if let Ok(hooks) = hooks {
            for hook in hooks {
                hook();
            }
        }

        let dirs = self.dirs.lock().map(|d| d.clone());
        if let Ok(dirs) = dirs {
            for dir in &dirs {
                crate::util::exe_dir::remove_dir_if_exists(dir);
            }
        }
    }
}

impl Drop for CleanupManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}
