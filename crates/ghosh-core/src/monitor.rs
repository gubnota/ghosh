//! Ephemeral monitoring; commands execute only after an explicit local action.
use crate::{contracts::*, ssh::NodeConnectionCommit, terminal::PtyRead};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct MonitorOpen {
    pub context: RevisionRequest,
    pub connection: Option<NodeConnectionCommit>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct MonitorSession {
    pub id: String,
    pub target: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
pub struct MonitorMetrics {
    pub cpu_percent: Option<f64>,
    pub memory_percent: Option<f64>,
    pub disk_percent: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct MonitorPacket {
    pub output: String,
    pub truncated: bool,
    pub running: bool,
    pub exit_code: Option<u32>,
    pub metrics: MonitorMetrics,
}
/// One owner serializes two independent non-PTY command channels. Slot 0 is logs,
/// slot 1 is a short metric probe. Dropping closes both owned channels.
pub trait MonitorTransport: Send {
    fn start(&mut self, slot: usize, command: &str) -> Result<()>;
    fn poll(&mut self, slot: usize) -> Result<PtyRead>;
    fn stop(&mut self, slot: usize);
}
/// Percent values only; malformed, missing or impossible readings stay unavailable.
pub fn parse_metrics(source: &str) -> MonitorMetrics {
    let percentage = |key: &str| {
        source.lines().find_map(|line| {
            let value = line.strip_prefix(key)?.parse::<f64>().ok()?;
            (value.is_finite() && (0.0..=100.0).contains(&value)).then_some(value)
        })
    };
    MonitorMetrics {
        cpu_percent: percentage("cpu="),
        memory_percent: percentage("memory="),
        disk_percent: percentage("disk="),
    }
}
// Linux CPU samples use /proc deltas; Darwin uses top's machine-wide CPU summary.
// No installation, sudo, credentials or files are involved.
pub const METRICS_COMMAND: &str = r#"case "$(uname -s)" in
Linux)
  a=$(awk '/^cpu /{idle=$5+$6; sum=0; for(i=2;i<=9;i++)sum+=$i; print sum,idle; exit}' /proc/stat)
  sleep 1
  b=$(awk '/^cpu /{idle=$5+$6; sum=0; for(i=2;i<=9;i++)sum+=$i; print sum,idle; exit}' /proc/stat)
  printf '%s %s\n' "$a" "$b" | awk '$3>$1{printf "cpu=%.2f\n",100*(1-($4-$2)/($3-$1))}'
  awk '/MemTotal:/{t=$2}/MemAvailable:/{a=$2}END{if(t>0)printf "memory=%.2f\n",100*(t-a)/t}' /proc/meminfo
  ;;
Darwin)
  LC_ALL=C top -l 1 -n 0 | awk '/CPU usage:/{gsub(/%/,"",$3);gsub(/%/,"",$5);printf "cpu=%.2f\n",$3+$5}'
  total=$(sysctl -n hw.memsize)
  vm_stat | awk -v total="$total" '/page size of/{for(i=1;i<=9;i++)if($i=="of")p=$(i+1)}/Pages active:|Pages wired down:|Pages occupied by compressor:/{gsub(/\./,"",$NF);used+=$NF}END{if(total>0&&p>0)printf "memory=%.2f\n",100*used*p/total}'
  ;;
esac
LC_ALL=C df -Pk / | awk 'NR==2{gsub(/%/,"",$5);print "disk=" $5}'
"#;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unavailable_and_invalid_metrics() {
        let m = parse_metrics("cpu=23.4\nmemory=NaN\ndisk=101\n");
        assert_eq!(m.cpu_percent, Some(23.4));
        assert_eq!(m.memory_percent, None);
        assert_eq!(m.disk_percent, None);
        assert_eq!(parse_metrics("permission denied").cpu_percent, None);
    }
}
