//! SigmaOS-power-manager — wlogout/systemd-logind-style power CLI.
//!
//! Usage:
//!   sigmaos-power status
//!   sigmaos-power menu            (list actions)
//!   sigmaos-power <action> [--yes]
//!     actions: suspend hibernate hybrid-sleep reboot shutdown logout
//!              lock dpms-off dpms-on cancel-shutdown
//!   sigmaos-power battery
//!   sigmaos-power inhibit <who> <why> | uninhibit <who>

use sigmaos_power_manager::{BatteryState, SigmaOSPower, PowerAction};
use std::process::exit;

fn parse_action(s: &str) -> Option<PowerAction> {
    Some(match s {
        "suspend" => PowerAction::Suspend,
        "hibernate" => PowerAction::Hibernate,
        "hybrid-sleep" => PowerAction::HybridSleep,
        "reboot" => PowerAction::Reboot,
        "shutdown" => PowerAction::Shutdown,
        "logout" => PowerAction::Logout,
        "lock" => PowerAction::Lock,
        "dpms-off" => PowerAction::DpmsOff,
        "dpms-on" => PowerAction::DpmsOn,
        "cancel-shutdown" => PowerAction::CancelShutdown,
        _ => return None,
    })
}

fn usage() -> ! {
    eprintln!("usage: sigmaos-power <status|menu|battery|inhibit|uninhibit|suspend|hibernate|hybrid-sleep|reboot|shutdown|logout|lock|dpms-off|dpms-on|cancel-shutdown>");
    exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() { usage(); }
    let mut p = SigmaOSPower::new();
    // demo battery state
    p.battery.percent = 82;
    p.battery.state = BatteryState::Discharging;
    p.battery.time_to_empty_min = Some(240);
    let cmd = args[0].as_str();
    let rest = &args[1..];

    let result: Result<(), String> = match cmd {
        "status" => { println!("{}", p.status_line()); Ok(()) }
        "battery" => { println!("{}", p.battery_line()); Ok(()) }
        "menu" => {
            for a in [PowerAction::Suspend, PowerAction::Hibernate, PowerAction::HybridSleep,
                      PowerAction::Lock, PowerAction::Logout, PowerAction::Reboot,
                      PowerAction::Shutdown, PowerAction::DpmsOff] {
                println!("{:<14} {}", a.label(), if a.destructive() { "(confirm)" } else { "" });
            }
            Ok(())
        }
        "inhibit" => {
            let who = rest.first().map(|s| s.as_str()).unwrap_or_else(|| usage());
            let why = rest.get(1).map(|s| s.as_str()).unwrap_or("");
            p.inhibit(who, why); println!("inhibited by {who}"); Ok(())
        }
        "uninhibit" => {
            let who = rest.first().map(|s| s.as_str()).unwrap_or_else(|| usage());
            p.uninhibit(who); println!("uninhibited {who}"); Ok(())
        }
        _ => {
            let action = match parse_action(cmd) {
                Some(a) => a,
                None => {
                    eprintln!("sigmaos-power: unknown action '{cmd}'");
                    exit(1);
                }
            };
            let yes = rest.iter().any(|s| s == "--yes" || s == "-y");
            match p.request(action, yes) {
                Ok(Some(pending)) => {
                    if pending.confirmed {
                        let done = p.execute_pending();
                        println!("executed {}", done.map(|a| a.label()).unwrap_or("?"));
                    } else {
                        println!("pending confirmation: {} (re-run with --yes)", pending.action.label());
                    }
                    Ok(())
                }
                Ok(None) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        }
    };

    if let Err(e) = result {
        eprintln!("sigmaos-power: {e}");
        exit(1);
    }
}
