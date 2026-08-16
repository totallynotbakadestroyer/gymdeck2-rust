use std::env;
use std::ffi::c_void;
use std::fs;
use std::io::{self, BufWriter, Write};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use serde::Deserialize;

const NUM_PHYSICAL_CORES: usize = 4;
const NUM_SAMPLES: usize = 10;
const NUM_LOGICAL_CPUS: usize = NUM_PHYSICAL_CORES * 2;

type RyzenAccess = *mut c_void;

#[link(name = "ryzenadj")]
extern "C" {
    fn init_ryzenadj() -> RyzenAccess;
    fn cleanup_ryzenadj(ry: RyzenAccess);
    fn set_coper(ry: RyzenAccess, value: u32) -> i32;
}

#[derive(Deserialize)]
struct ManualPoint {
    point: i32,
    value: i32,
}

struct ManualPointsProfile {
    frequency: i32,
    points: Vec<ManualPoint>,
}

struct ManualPointsCollection {
    profiles: Vec<ManualPointsProfile>,
}

struct CpuStats {
    logical_cpu_usage: [f32; NUM_LOGICAL_CPUS],
    prev_idle: [i64; NUM_LOGICAL_CPUS],
    prev_total: [i64; NUM_LOGICAL_CPUS],
    stat_updated: bool,
}

impl CpuStats {
    fn new() -> Self {
        CpuStats {
            logical_cpu_usage: [0.0; NUM_LOGICAL_CPUS],
            prev_idle: [0; NUM_LOGICAL_CPUS],
            prev_total: [0; NUM_LOGICAL_CPUS],
            stat_updated: false,
        }
    }

    fn read_proc_stat_once(&mut self) {
        if self.stat_updated {
            return;
        }

        let contents = match fs::read_to_string("/proc/stat") {
            Ok(contents) => contents,
            Err(err) => {
                eprintln!("Failed to open /proc/stat: {}", err);
                return;
            }
        };

        let mut line_num = 0;

        for line in contents.lines() {
            if line.starts_with("cpu") {
                if line_num == 0 {
                    line_num += 1;
                    continue;
                }

                let core_id = line_num - 1;
                if core_id >= NUM_LOGICAL_CPUS {
                    break;
                }

                let fields: Vec<i64> = line
                    .split_whitespace()
                    .skip(1)
                    .take(8)
                    .filter_map(|field| field.parse().ok())
                    .collect();

                if let [user, nice, system, idle, iowait, irq, softirq, steal] = fields[..] {
                    let idle_time = idle + iowait;
                    let non_idle_time = user + nice + system + irq + softirq + steal;
                    let total_time = idle_time + non_idle_time;

                    let total_diff = total_time - self.prev_total[core_id];
                    let idle_diff = idle_time - self.prev_idle[core_id];

                    self.prev_total[core_id] = total_time;
                    self.prev_idle[core_id] = idle_time;

                    if total_diff <= 0 {
                        self.logical_cpu_usage[core_id] = 0.0;
                    } else {
                        self.logical_cpu_usage[core_id] =
                            (total_diff - idle_diff) as f32 / total_diff as f32 * 100.0;
                    }
                }

                line_num += 1;
            } else if line_num > NUM_LOGICAL_CPUS {
                break;
            }
        }

        self.stat_updated = true;
    }

    fn get_cpu_usage(&mut self, core_id: usize) -> f32 {
        if !self.stat_updated {
            self.read_proc_stat_once();
        }
        if core_id >= NUM_LOGICAL_CPUS {
            return 0.0;
        }
        self.logical_cpu_usage[core_id]
    }
}

fn set_unsafe_coper(value: u32, ryzenadj: RyzenAccess) {
    if ryzenadj.is_null() {
        eprintln!("Unable to initialize RyzenAdj");
        exit(1);
    }

    if unsafe { set_coper(ryzenadj, value) } != 0 {
        println!("Error setting coper value for core {}", value);
        eprintln!("Error setting unsafe coper value");
    }
}

fn calculate_hex_value(core: i32, value: i32) -> u32 {
    let core_shifted = core as u32 * 0x100000;
    let magnitude = value.wrapping_neg() as u32 & 0xFFFFF;

    core_shifted + magnitude
}

fn get_current_frequency(physical_core: usize) -> i32 {
    let path = format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq",
        physical_core * 2
    );
    let freq_khz: i32 = match fs::read_to_string(&path) {
        Ok(contents) => match contents.trim().parse() {
            Ok(freq_khz) => freq_khz,
            Err(_) => return -1,
        },
        Err(_) => return -1,
    };
    let freq_mhz = freq_khz / 1000;
    (freq_mhz + 50) / 100 * 100
}

fn print_logo() {
    println!(" ________      ___    ___ _____ ______   ________  _______   ________  ___  __      _______     ");
    println!("|\\   ____\\    |\\  \\  /  /|\\   _ \\  _   \\|\\   ___ \\|\\  ___ \\ |\\   ____\\|\\  \\|\\  \\   /  ___  \\    ");
    println!("\\ \\  \\___|    \\ \\  \\/  / | \\  \\\\\\__\\ \\  \\ \\  \\_|\\ \\ \\   __/|\\ \\  \\___|\\ \\  \\/  /|_/__/|_/  /|   ");
    println!(" \\ \\  \\  ___   \\ \\    / / \\ \\  \\\\\\|__| \\  \\ \\  \\ \\\\ \\ \\  \\_|/_\\ \\  \\    \\ \\   ___  \\__|//  / /   ");
    println!("  \\ \\  \\|\\  \\   \\/  /  /   \\ \\  \\    \\ \\  \\ \\  \\_\\\\ \\ \\  \\_|\\ \\ \\  \\____\\ \\  \\\\ \\  \\  /  /_/__  ");
    println!("   \\ \\_______\\__/  / /      \\ \\__\\    \\ \\__\\ \\_______\\ \\_______\\ \\_______\\ \\__\\\\ \\__\\|\\________\\");
    println!("    \\|_______|\\___/ /        \\|__|     \\|__|\\|_______|\\|_______|\\|_______|\\|__| \\|__| \\|_______|");
    println!("             \\|___|/                                                                           ");
    let _ = io::stdout().flush();
}

fn print_usage(prog_name: &str) {
    eprintln!("Usage: {} sample_interval [manual_points_core_0] ... [manual_points_core_3]", prog_name);
    eprintln!("Sample Interval: microseconds");
    eprintln!("Manual Points (object format): {{frequency: [{{point: int, value: int}}, ...], ...}}");
}

fn parse_manual_points_object(json_str: &str) -> ManualPointsCollection {
    let mut collection = ManualPointsCollection { profiles: Vec::new() };

    let json: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(json) => json,
        Err(err) => {
            eprintln!("Bad JSON {}", err);
            return collection;
        }
    };

    let object = match json.as_object() {
        Some(object) => object,
        None => {
            eprintln!("Not an object, what are you even trying to do?");
            return collection;
        }
    };

    for (key, item) in object {
        let mut profile = ManualPointsProfile {
            frequency: key.trim().parse().unwrap_or(0),
            points: Vec::new(),
        };
        if let Some(array) = item.as_array() {
            for pt in array {
                if let Ok(point) = serde_json::from_value::<ManualPoint>(pt.clone()) {
                    profile.points.push(point);
                }
            }
        }
        collection.profiles.push(profile);
    }

    collection
}

#[allow(unreachable_code)]
fn main() {
    print_logo();
    sleep(Duration::from_micros(1_000_000));
    let ryzenadj = unsafe { init_ryzenadj() };

    let argv: Vec<String> = env::args().collect();
    if argv.len() < NUM_PHYSICAL_CORES + 2 {
        print_usage(&argv[0]);
        exit(1);
    }

    let sample_interval: i32 = argv[1].trim().parse().unwrap_or(0);
    if sample_interval <= 0 {
        eprintln!("Invalid sample interval: {}", sample_interval);
        exit(1);
    }

    let manual_collection: Vec<ManualPointsCollection> = (0..NUM_PHYSICAL_CORES)
        .map(|i| parse_manual_points_object(&argv[2 + i]))
        .collect();

    let mut stats = CpuStats::new();
    let mut core_loads = [[0.0f32; NUM_SAMPLES]; NUM_PHYSICAL_CORES];
    let mut sample_count = [0usize; NUM_PHYSICAL_CORES];
    let mut last_applied_steps = [-1i32; NUM_PHYSICAL_CORES];

    loop {
        stats.stat_updated = false;
        for i in 0..NUM_PHYSICAL_CORES {
            let load1 = stats.get_cpu_usage(i * 2);
            let load2 = stats.get_cpu_usage(i * 2 + 1);
            let combined_load = (load1 + load2) / 2.0;
            core_loads[i][sample_count[i] % NUM_SAMPLES] = combined_load;
            sample_count[i] += 1;
        }

        let stdout = io::stdout();
        let mut out = BufWriter::new(stdout.lock());
        let _ = write!(out, "\x1b[H\x1b[J");

        for i in 0..NUM_PHYSICAL_CORES {
            let samples_to_consider = sample_count[i].min(NUM_SAMPLES);
            let mut total_load = 0.0;
            for j in 0..samples_to_consider {
                total_load += core_loads[i][j];
            }
            let mut average_load = 0.0f32;
            if samples_to_consider > 0 {
                average_load = total_load / samples_to_consider as f32;
            }

            let current_freq = get_current_frequency(i);
            let mut selected_profile: Option<&ManualPointsProfile> = None;
            let mut min_diff = 100000;
            for profile in &manual_collection[i].profiles {
                let diff = (profile.frequency - current_freq).abs();
                if diff < min_diff {
                    min_diff = diff;
                    selected_profile = Some(profile);
                }
            }

            let mut curve_optimizer_step = 0;
            if let Some(profile) = selected_profile {
                if !profile.points.is_empty() {
                    curve_optimizer_step = profile.points[0].value;
                    for point in &profile.points {
                        if average_load >= point.point as f32 {
                            curve_optimizer_step = point.value;
                        } else {
                            break;
                        }
                    }
                }
            }

            if curve_optimizer_step != last_applied_steps[i] {
                let coper_value = calculate_hex_value(i as i32, curve_optimizer_step);
                set_unsafe_coper(coper_value, ryzenadj);
                last_applied_steps[i] = curve_optimizer_step;
            }
            let _ = write!(out, "Physical Core {}: Freq: {} MHz, Average Load: {:.2}%, Curve Optimizer Step: {} | ", i + 1, current_freq, average_load, curve_optimizer_step);
        }
        let _ = writeln!(out);
        let _ = out.flush();
        drop(out);
        sleep(Duration::from_micros(sample_interval as u64));
    }

    unsafe { cleanup_ryzenadj(ryzenadj) };
}
