use std::net::Ipv4Addr;

#[derive(Debug, Clone, serde::Serialize)]
pub struct NetworkInterface {
    pub name: String,
    pub addr: Option<Ipv4Addr>,
    pub is_zt: bool,
}

pub fn list() -> Vec<NetworkInterface> {
    // Читаем /proc/net/if_inet6 и /proc/net/dev для базового списка
    // В PART 2 будет расширено через nix crate
    let proc_dev = std::fs::read_to_string("/proc/net/dev").unwrap_or_default();
    let mut ifaces = Vec::new();
    for line in proc_dev.lines().skip(2) {
        let name = line.split(':').next().unwrap_or("").trim().to_string();
        if name.is_empty() {
            continue;
        }
        let is_zt = name.starts_with("zt");
        ifaces.push(NetworkInterface {
            name,
            addr: None,
            is_zt,
        });
    }
    ifaces
}
