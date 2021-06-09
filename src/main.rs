use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

use crate::config::read_config;
use crate::macos::OSXStatusBar;

mod config;
mod macos;

use chrono::prelude::*;
use chrono_tz::Tz;
use notify::{watcher, RecursiveMode, Watcher};
use run_script::ScriptOptions;

// mac structs copied from https://github.com/sim-o/workstatus
pub type NSCallback = Box<dyn Fn(u64, &Sender<String>)>;

fn main() {
    let args: Vec<String> = env::args().collect();

    let options = ScriptOptions::new();

    if args.len() > 1 && args[1] == "install" {
        let script = r#"
DST="/Applications"
APPDIR="Bartime.app"

rm -rf "$DST/$APPDIR"
mkdir "$DST/$APPDIR/"
mkdir "$DST/$APPDIR/Contents/"
mkdir "$DST/$APPDIR/Contents/Resources/"
mkdir "$DST/$APPDIR/Contents/MacOS/"

cp -a ~/.cargo/bin/bartime "$DST/$APPDIR/Contents/MacOS/"
/usr/bin/strip -u -r "$DST/$APPDIR/Contents/MacOS/bartime"

cat > "$DST/$APPDIR/Contents/Info.plist" << EOF
{{
   CFBundleName = bartime;
   CFBundleDisplayName = Bartime;
   CFBundleIdentifier = "com.drbh.bartime";
   CFBundleExecutable = bartime;
   CFBundleIconFile = "bartime.icns";
   CFBundleVersion = "0.0.2";
   CFBundleInfoDictionaryVersion = "6.0";
   CFBundlePackageType = APPL;
   CFBundleSignature = xxxx;
   LSMinimumSystemVersion = "10.10.0";
}}
EOF
         "#;
        let (_code, _output, _error) = run_script::run(&script, &args, &options).unwrap();
        println!("Installed bartime.app at /Applications/Bartime");
        println!("Configuration file at ~/.bartime/config.toml");
        return;
    }

    let config_path = "/Users/drbh/.bartime/config.toml";

    if Path::new(config_path).exists() {
        // fs::remove_file(file).unwrap();
    } else {
        fs::create_dir_all("/Users/drbh/.bartime").expect("failed to make dir");

        let _file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(config_path);

        let data = r#"
[[location]]
    name = "NYC 🗽"
    tz = "America/New_York"
        "#;
        fs::write(config_path, data).expect("Unable to write file");
    }

    let config = read_config(config_path).expect("error reading config.toml");
    let amlocations = Arc::new(Mutex::new(config.location));

    let file_change_reset_locations = Arc::clone(&amlocations);
    let interval_reset_locations = Arc::clone(&amlocations);
    let force_reset_locations = Arc::clone(&amlocations);

    let (tx, rx) = channel();

    thread::spawn(move || loop {
        match rx.recv() {
            Ok(_event) => {
                let _config = read_config(config_path).expect("error reading config.toml");
                *file_change_reset_locations.lock().unwrap() = _config.location
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    });

    let mut watcher = watcher(tx, Duration::from_millis(2_000)).unwrap();

    watcher
        .watch(config_path, RecursiveMode::Recursive)
        .unwrap();

    let (tx_query, rx_query) = channel::<String>();

    let mut status_bar = {
        let mut status_bar = OSXStatusBar::new(&String::from(""), tx_query.clone());
        {
            let cb: NSCallback = Box::new(move |_sender, tx| {
                tx.send("manual".to_string())
                    .expect("manual refresh send failed");

                // force a reread
                let _config = read_config(config_path).expect("error reading config.toml");
                *force_reset_locations.lock().unwrap() = _config.location;
            });
            let _ = status_bar.add_item(None, "Refresh", cb, false);
        }
        {
            let cb: NSCallback = Box::new(move |_sender, _tx| {
                exit(0);
            });
            let _ = status_bar.add_item(None, "Quit", cb, false);
        }

        let tx_query_manual = tx_query; //.clone();
        thread::spawn(move || loop {
            tx_query_manual
                .send("interval".to_string())
                .expect("interval send failed");
            thread::sleep(Duration::from_millis(3_000));
        });
        status_bar
    };

    let rx = {
        let (tx, rx) = channel::<String>();
        let stopper = status_bar.stopper();
        thread::spawn(move || {
            for _reason in rx_query.iter() {
                let times = &*interval_reset_locations.lock().unwrap();
                let mut bar_text = String::from("");
                for item in times.iter() {
                    let abbr = item.name.clone();
                    let location = item.tz.clone();
                    let time = get_remote_time(&location);
                    bar_text.push_str(&abbr);
                    bar_text.push(' ');
                    bar_text.push_str(&time);
                    bar_text.push_str("       ");
                }
                tx.send(bar_text).expect("worker send failed");
                stopper.stop();
            }
        });
        rx
    };

    loop {
        status_bar.run(true);
        while let Ok(title) = rx.try_recv() {
            status_bar.set_title(title.as_str());
        }
    }
}

fn get_remote_time(timeloc: &str) -> String {
    let tz: Tz = timeloc.parse().unwrap();
    let utc: DateTime<Utc> = Utc::now();
    let remote_time = utc.with_timezone(&tz);
    let subset_time = remote_time.format("%a %H:%M").to_string();
    subset_time
}
