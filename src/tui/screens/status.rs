//! Status screen - Real-time system health dashboard

use crate::status::SystemStatus;
use crate::tui::{
    state::AppState,
    theme::Styles,
    widgets::{
        logo::{render_logo, render_tagline, LOGO_WITH_TAGLINE_HEIGHT},
        shortcuts::{get_shortcuts, render_shortcuts},
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Color,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app_state: &AppState) {
    let area = f.area();

    let is_small = area.height < 20 || area.width < 60;
    let shortcuts_height = if is_small { 2 } else { 3 };

    let header_height = LOGO_WITH_TAGLINE_HEIGHT;

    // Ensure we have minimum space
    let min_content_height = 20;
    let min_total_height = header_height + min_content_height + shortcuts_height;

    if area.height < min_total_height || area.width < 60 {
        let msg = Paragraph::new("Terminal too small. Please resize to at least 60x25")
            .style(Styles::warning())
            .alignment(Alignment::Center);
        f.render_widget(msg, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(min_content_height),
            Constraint::Length(shortcuts_height),
        ])
        .split(area);

    render_header(f, chunks[0], is_small);
    render_content(f, chunks[1], app_state, is_small);

    let shortcuts = get_shortcuts(&app_state.screen, Some(app_state));
    render_shortcuts(f, chunks[2], &shortcuts);
}

fn render_header(f: &mut Frame, area: Rect, _is_small: bool) {
    render_logo(f, area);
    render_tagline(f, area);
}

fn render_content(f: &mut Frame, area: Rect, app_state: &AppState, _is_small: bool) {
    if let crate::tui::state::Screen::Status {
        status,
        last_refresh,
    } = &app_state.screen
    {
        // Header with health score and live indicator
        let header_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Health header (2 lines)
                Constraint::Min(1),    // Main content
            ])
            .split(area);

        render_status_header_with_indicator(f, header_chunks[0], status, last_refresh);

        // Main content area
        render_status_dashboard(f, header_chunks[1], status);
    }
}

fn render_status_header_with_indicator(
    f: &mut Frame,
    area: Rect,
    status: &SystemStatus,
    last_refresh: &std::time::Instant,
) {
    let health_indicator = match status.health_score {
        80..=100 => ("●", Color::Green),
        60..=79 => ("○", Color::Yellow),
        40..=59 => ("◐", Color::Magenta),
        _ => ("◯", Color::Red),
    };

    // Simple live indicator - just show if it's live or not
    let elapsed_ms = last_refresh.elapsed().as_millis();
    let live_indicator = if elapsed_ms < 3000 {
        "● Live"
    } else {
        "○ Updated"
    };

    let header_style = match status.health_score {
        80..=100 => Styles::success(),
        60..=79 => Styles::warning(),
        _ => Styles::error(),
    };

    // Split into two lines
    let lines = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // Line 1: Health status with live indicator
    let health_text = format!(
        "Health status: {} {}",
        health_indicator.0, status.health_score
    );
    let health_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(lines[0]);

    let health_para = Paragraph::new(health_text)
        .style(header_style)
        .alignment(Alignment::Left);
    f.render_widget(health_para, health_chunks[0]);

    let live_para = Paragraph::new(live_indicator)
        .style(Styles::success())
        .alignment(Alignment::Right);
    f.render_widget(live_para, health_chunks[1]);

    // Line 2: Device information with uptime
    let uptime_str = format_uptime(status.hardware.uptime_seconds);
    let device_text = format!(
        "{} · {} · {:.1}GB · {} · Uptime: {}",
        status.hardware.device_name,
        status.hardware.cpu_model,
        status.hardware.total_memory_gb,
        status.hardware.os_name,
        uptime_str
    );
    let device_para = Paragraph::new(device_text)
        .style(Styles::secondary())
        .alignment(Alignment::Left);
    f.render_widget(device_para, lines[1]);
}

fn render_status_dashboard(f: &mut Frame, area: Rect, status: &SystemStatus) {
    // Main layout: top section (columns) and bottom section (network/processes)
    let main_sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(20), // Top section (CPU/Memory/Disk/Power) - increased for process count
            Constraint::Length(1),  // Spacing
            Constraint::Length(7),  // Network section (ensure it's always visible)
            Constraint::Length(1),  // Spacing
            Constraint::Min(5),     // Processes section
        ])
        .split(area);

    // Top section: Two column layout
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_sections[0]);

    // Left column: CPU and Memory
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11), // CPU (increased for process count)
            Constraint::Length(1),  // Spacing
            Constraint::Length(6),  // Memory (reduced since we removed redundant Free/Avail)
        ])
        .split(columns[0]);

    render_cpu_section(f, left_chunks[0], status);
    render_memory_section(f, left_chunks[2], status);

    // Right column: Disk and Power
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Disk
            Constraint::Length(1),  // Spacing
            Constraint::Length(12), // Power (increased to accommodate new fields)
        ])
        .split(columns[1]);

    render_disk_section(f, right_chunks[0], status);
    render_power_section(f, right_chunks[2], status);

    // Network and Processes in bottom sections
    render_network_section(f, main_sections[2], status);
    render_processes_section(f, main_sections[4], status);
}

fn render_cpu_section(f: &mut Frame, area: Rect, status: &SystemStatus) {
    let cpu_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Styles::border())
        .title("⚙ CPU");

    let inner = cpu_block.inner(area);
    f.render_widget(cpu_block, area);

    let lines = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Total
            Constraint::Length(1), // Load
            Constraint::Length(1), // Processor brand
            Constraint::Length(1), // Frequency/Vendor
            Constraint::Length(1), // Processes
            Constraint::Length(1), // Spacing
            Constraint::Min(1),    // Cores
        ])
        .split(inner);

    // Total CPU usage
    let total_bar = create_progress_bar(status.cpu.total_usage / 100.0, 20);
    let total_text = format!("Total   {}  {:.1}%", total_bar, status.cpu.total_usage);
    let total_para = Paragraph::new(total_text).style(Styles::primary());
    f.render_widget(total_para, lines[0]);

    // Load averages
    let load_text = format!(
        "Load    {:.2} / {:.2} / {:.2} ({} cores)",
        status.cpu.load_avg_1min,
        status.cpu.load_avg_5min,
        status.cpu.load_avg_15min,
        status.cpu.cores.len()
    );
    let load_para = Paragraph::new(load_text).style(Styles::secondary());
    f.render_widget(load_para, lines[1]);

    // Processor brand (first line)
    let brand_text = format!("Proc    {}", status.cpu.brand);
    let brand_para = Paragraph::new(brand_text).style(Styles::secondary());
    f.render_widget(brand_para, lines[2]);

    // Frequency and vendor info (second line)
    let freq_text = if let Some(freq_mhz) = status.cpu.frequency_mhz {
        let freq_ghz = freq_mhz as f64 / 1000.0;
        format!("Freq    {:.2} GHz · {}", freq_ghz, status.cpu.vendor_id)
    } else {
        format!("Vendor  {}", status.cpu.vendor_id)
    };
    let freq_para = Paragraph::new(freq_text).style(Styles::secondary());
    f.render_widget(freq_para, lines[3]);

    // Process count
    let proc_text = format!("Procs   {}", status.cpu.process_count);
    let proc_para = Paragraph::new(proc_text).style(Styles::secondary());
    f.render_widget(proc_para, lines[4]);

    // Show first few cores
    let core_count = (lines[5].height as usize)
        .min(status.cpu.cores.len())
        .min(3);
    for (i, core) in status.cpu.cores.iter().take(core_count).enumerate() {
        if i < lines[5].height as usize {
            let core_bar = create_progress_bar(core.usage / 100.0, 20);
            let core_text = format!("Core {}  {}  {:.1}%", core.id + 1, core_bar, core.usage);
            let core_para = Paragraph::new(core_text).style(Styles::secondary());
            let core_area = Rect {
                x: lines[5].x,
                y: lines[5].y + i as u16,
                width: lines[5].width,
                height: 1,
            };
            f.render_widget(core_para, core_area);
        }
    }
}

fn render_memory_section(f: &mut Frame, area: Rect, status: &SystemStatus) {
    let mem_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Styles::border())
        .title("▦ Memory");

    let inner = mem_block.inner(area);
    f.render_widget(mem_block, area);

    let lines = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Used
            Constraint::Length(1), // Total
            Constraint::Length(1), // Free/Available (combined)
            Constraint::Length(1), // Swap
        ])
        .split(inner);

    // Used memory
    let used_bar = create_progress_bar(status.memory.used_percent / 100.0, 20);
    let used_text = format!("Used    {}  {:.1}%", used_bar, status.memory.used_percent);
    let used_para = Paragraph::new(used_text).style(Styles::primary());
    f.render_widget(used_para, lines[0]);

    // Total memory
    let total_text = format!(
        "Total   {:.1} / {:.1} GB",
        status.memory.used_gb, status.memory.total_gb
    );
    let total_para = Paragraph::new(total_text).style(Styles::secondary());
    f.render_widget(total_para, lines[1]);

    // Free/Available memory (combined since they're usually the same)
    let free_text = format!("Free    {:.1} GB", status.memory.free_gb);
    let free_para = Paragraph::new(free_text).style(Styles::secondary());
    f.render_widget(free_para, lines[2]);

    // Swap/Page file memory
    if status.memory.swap_total_gb > 0.0 {
        let swap_bar = create_progress_bar(status.memory.swap_percent / 100.0, 20);
        let swap_text = format!(
            "Swap    {}  {:.1}% ({:.1} / {:.1} GB)",
            swap_bar,
            status.memory.swap_percent,
            status.memory.swap_used_gb,
            status.memory.swap_total_gb
        );
        let swap_para = Paragraph::new(swap_text).style(Styles::secondary());
        f.render_widget(swap_para, lines[3]);
    }
}

fn render_disk_section(f: &mut Frame, area: Rect, status: &SystemStatus) {
    let disk_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Styles::border())
        .title("▤ Disk");

    let inner = disk_block.inner(area);
    f.render_widget(disk_block, area);

    let lines = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Used
            Constraint::Length(1), // Free
            Constraint::Length(1), // Read
            Constraint::Length(1), // Write
        ])
        .split(inner);

    // Used disk
    let used_bar = create_progress_bar(status.disk.used_percent / 100.0, 20);
    let used_text = format!("Used    {}  {:.1}%", used_bar, status.disk.used_percent);
    let used_para = Paragraph::new(used_text).style(Styles::primary());
    f.render_widget(used_para, lines[0]);

    // Free disk
    let free_text = format!("Free    {:.1} GB", status.disk.free_gb);
    let free_para = Paragraph::new(free_text).style(Styles::secondary());
    f.render_widget(free_para, lines[1]);

    // Read speed
    let read_bar = create_speed_bar(status.disk.read_speed_mb / 100.0, 5);
    let read_text = format!(
        "Read    {}  {:.1} MB/s",
        read_bar, status.disk.read_speed_mb
    );
    let read_para = Paragraph::new(read_text).style(Styles::secondary());
    f.render_widget(read_para, lines[2]);

    // Write speed
    let write_bar = create_speed_bar(status.disk.write_speed_mb / 100.0, 5);
    let write_text = format!(
        "Write   {}  {:.1} MB/s",
        write_bar, status.disk.write_speed_mb
    );
    let write_para = Paragraph::new(write_text).style(Styles::secondary());
    f.render_widget(write_para, lines[3]);
}

fn render_power_section(f: &mut Frame, area: Rect, status: &SystemStatus) {
    let power_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Styles::border())
        .title("⚡ Power");

    let inner = power_block.inner(area);
    f.render_widget(power_block, area);

    if let Some(power) = &status.power {
        let mut constraints = vec![
            Constraint::Length(1), // Level
            Constraint::Length(1), // Status
            Constraint::Length(1), // Health
            Constraint::Length(1), // Cycles
        ];

        if power.time_to_empty_seconds.is_some() || power.time_to_full_seconds.is_some() {
            constraints.push(Constraint::Length(1)); // Time estimate
        }
        if power.voltage_volts.is_some() {
            constraints.push(Constraint::Length(1)); // Voltage
        }
        if power.energy_rate_watts.is_some() {
            constraints.push(Constraint::Length(1)); // Energy rate
        }
        if power.design_capacity_mwh.is_some() {
            constraints.push(Constraint::Length(1)); // Design Capacity
        }
        if power.full_charge_capacity_mwh.is_some() {
            constraints.push(Constraint::Length(1)); // Full Charge Capacity
        }

        let lines = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        let mut line_idx = 0;

        // Battery level - match Memory "Used" format (8-char label, 20-char bar, percentage)
        let level_bar = create_progress_bar(power.level_percent / 100.0, 20);
        let level_text = format!("Level   {}  {:.1}%", level_bar, power.level_percent);
        let level_para = Paragraph::new(level_text).style(Styles::primary());
        f.render_widget(level_para, lines[line_idx]);
        line_idx += 1;

        // Status - match Memory "Total" position
        let status_text = format!("Status  {}", power.status);
        let status_para = Paragraph::new(status_text).style(Styles::secondary());
        f.render_widget(status_para, lines[line_idx]);
        line_idx += 1;

        // Health - match Memory "Free" position
        let health_text = format!("Health  {}", power.health);
        let health_para = Paragraph::new(health_text).style(Styles::secondary());
        f.render_widget(health_para, lines[line_idx]);
        line_idx += 1;

        // Cycles - match Memory "Swap" position
        if let Some(cycles) = power.cycles {
            let cycles_text = format!("Cycles  {}", cycles);
            let cycles_para = Paragraph::new(cycles_text).style(Styles::secondary());
            f.render_widget(cycles_para, lines[line_idx]);
            line_idx += 1;
        } else {
            line_idx += 1;
        }

        // Time estimates
        if let Some(time_to_empty) = power.time_to_empty_seconds {
            let time_str = format_time(time_to_empty);
            let time_text = format!("Time    {} left", time_str);
            let time_para = Paragraph::new(time_text).style(Styles::secondary());
            f.render_widget(time_para, lines[line_idx]);
            line_idx += 1;
        } else if let Some(time_to_full) = power.time_to_full_seconds {
            let time_str = format_time(time_to_full);
            let time_text = format!("Time    {} to full", time_str);
            let time_para = Paragraph::new(time_text).style(Styles::secondary());
            f.render_widget(time_para, lines[line_idx]);
            line_idx += 1;
        }

        // Voltage
        if let Some(voltage) = power.voltage_volts {
            let voltage_text = format!("Volt    {:.2} V", voltage);
            let voltage_para = Paragraph::new(voltage_text).style(Styles::secondary());
            f.render_widget(voltage_para, lines[line_idx]);
            line_idx += 1;
        }

        // Energy rate
        if let Some(rate) = power.energy_rate_watts {
            let rate_text = format!("Power   {:.1} W", rate);
            let rate_para = Paragraph::new(rate_text).style(Styles::secondary());
            f.render_widget(rate_para, lines[line_idx]);
            line_idx += 1;
        }

        // Design Capacity
        if let Some(design_cap) = power.design_capacity_mwh {
            let design_text = format!("Design  {:.0} mWh", design_cap);
            let design_para = Paragraph::new(design_text).style(Styles::secondary());
            f.render_widget(design_para, lines[line_idx]);
            line_idx += 1;
        }

        // Full Charge Capacity
        if let Some(full_cap) = power.full_charge_capacity_mwh {
            let full_text = format!("Full    {:.0} mWh", full_cap);
            let full_para = Paragraph::new(full_text).style(Styles::secondary());
            f.render_widget(full_para, lines[line_idx]);
        }
    } else {
        // No battery - show plugged in status
        let text = Paragraph::new("Status  Plugged In")
            .style(Styles::secondary())
            .alignment(Alignment::Left);
        f.render_widget(text, inner);
    }
}

fn render_network_section(f: &mut Frame, area: Rect, status: &SystemStatus) {
    let network_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Styles::border())
        .title("⇅ Network");

    let inner = network_block.inner(area);
    f.render_widget(network_block, area);

    // Find the primary interface (one with real IPs, prefer IPv4 192.x.x.x, then fe80, then any)
    let primary_iface = status
        .network_interfaces
        .iter()
        .find(|iface| iface.ip_addresses.iter().any(|ip| ip.starts_with("192.")))
        .or_else(|| {
            status
                .network_interfaces
                .iter()
                .find(|iface| iface.ip_addresses.iter().any(|ip| ip.starts_with("fe80:")))
        })
        .or_else(|| {
            status
                .network_interfaces
                .iter()
                .find(|iface| !iface.ip_addresses.is_empty())
        })
        .or_else(|| status.network_interfaces.first());

    // Collect IPs: prefer IPv4 192.x.x.x, then fe80, then others
    let mut ipv4_192 = Vec::new();
    let mut ipv6_fe80 = Vec::new();

    if let Some(iface) = primary_iface {
        for ip in &iface.ip_addresses {
            if ip.starts_with("192.") {
                ipv4_192.push(ip.clone());
            } else if ip.starts_with("fe80:") {
                ipv6_fe80.push(ip.clone());
            }
        }
    }

    // Always show at least download/upload, even if no interfaces found
    let mut constraints = vec![
        Constraint::Length(1), // Download
        Constraint::Length(1), // Upload
    ];

    // Show connection status and type if available
    if let Some(iface) = primary_iface {
        if iface.is_up || !iface.ip_addresses.is_empty() {
            constraints.push(Constraint::Length(1)); // Connection status/type
        }
    }

    if status.network.proxy.is_some() {
        constraints.push(Constraint::Length(1)); // Proxy
    }

    // Show MAC if available and valid
    if let Some(iface) = primary_iface {
        if let Some(ref mac) = iface.mac_address {
            if !mac.starts_with("00:00:00:00:00:00") {
                constraints.push(Constraint::Length(1)); // MAC
            }
        }
    }

    // Show IPs
    if !ipv4_192.is_empty() {
        constraints.push(Constraint::Length(1)); // IPv4
    }
    if !ipv6_fe80.is_empty() {
        constraints.push(Constraint::Length(1)); // IPv6 fe80
    }

    // If no IPs found, show a message
    if ipv4_192.is_empty() && ipv6_fe80.is_empty() && primary_iface.is_none() {
        constraints.push(Constraint::Length(1)); // No network message
    }

    let lines = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let mut line_idx = 0;

    // Download
    let down_bar = create_speed_bar(status.network.download_mb / 10.0, 5);
    let down_text = format!(
        "Down    {}  {:.1} MB/s",
        down_bar, status.network.download_mb
    );
    let down_para = Paragraph::new(down_text).style(Styles::secondary());
    f.render_widget(down_para, lines[line_idx]);
    line_idx += 1;

    // Upload
    let up_bar = create_speed_bar(status.network.upload_mb / 10.0, 5);
    let up_text = format!("Up      {}  {:.1} MB/s", up_bar, status.network.upload_mb);
    let up_para = Paragraph::new(up_text).style(Styles::secondary());
    f.render_widget(up_para, lines[line_idx]);
    line_idx += 1;

    // Connection status and type
    if let Some(iface) = primary_iface {
        if iface.is_up || !iface.ip_addresses.is_empty() {
            let conn_status = if iface.is_up || !iface.ip_addresses.is_empty() {
                "Connected"
            } else {
                "Disconnected"
            };
            let conn_type = iface.connection_type.as_deref().unwrap_or("Unknown");

            let conn_text = format!("Status  {} · {}", conn_status, conn_type);
            let conn_para = Paragraph::new(conn_text).style(Styles::secondary());
            f.render_widget(conn_para, lines[line_idx]);
            line_idx += 1;
        }
    }

    // Proxy
    if let Some(proxy) = &status.network.proxy {
        let proxy_text = format!("Proxy   {}", proxy);
        let proxy_para = Paragraph::new(proxy_text).style(Styles::secondary());
        f.render_widget(proxy_para, lines[line_idx]);
        line_idx += 1;
    }

    // Show MAC address (if valid)
    if let Some(iface) = primary_iface {
        if let Some(ref mac) = iface.mac_address {
            if !mac.starts_with("00:00:00:00:00:00") {
                let mac_text = format!("MAC     {}", mac);
                let mac_para = Paragraph::new(mac_text).style(Styles::secondary());
                f.render_widget(mac_para, lines[line_idx]);
                line_idx += 1;
            }
        }
    }

    // Show IPv4 192.x.x.x addresses
    if !ipv4_192.is_empty() {
        let ip_text = format!("IPv4    {}", ipv4_192[0]);
        let ip_para = Paragraph::new(ip_text).style(Styles::secondary());
        f.render_widget(ip_para, lines[line_idx]);
        line_idx += 1;
    }

    // Show IPv6 fe80 addresses
    if !ipv6_fe80.is_empty() {
        let ip_text = format!("IPv6    {}", ipv6_fe80[0]);
        let ip_para = Paragraph::new(ip_text).style(Styles::secondary());
        f.render_widget(ip_para, lines[line_idx]);
    } else if ipv4_192.is_empty() && primary_iface.is_none() {
        // Show message if no network interfaces found
        let msg_text = "No active network";
        let msg_para = Paragraph::new(msg_text).style(Styles::secondary());
        f.render_widget(msg_para, lines[line_idx]);
    }
}

fn render_processes_section(f: &mut Frame, area: Rect, status: &SystemStatus) {
    let processes_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Styles::border())
        .title("▶ Processes");

    let inner = processes_block.inner(area);
    f.render_widget(processes_block, area);

    // Two column layout for processes
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // Helper to format a process entry with consistent alignment
    // Format: "name         pid  bar cpu%  mem"
    // Compact layout with fixed widths for alignment
    let format_process = |proc: &crate::status::ProcessInfo| -> String {
        // Fixed name width of 14 chars for compact display
        let name = if proc.name.len() > 14 {
            format!("{}…", &proc.name[..13])
        } else {
            proc.name.clone()
        };

        let proc_bar = create_mini_bar(proc.cpu_usage / 100.0, 6);

        // Compact format: name(14) pid(5) bar(6) cpu(5) mem(5)
        format!(
            "{:<14} {:>5} {} {:>4.1}% {:>4}M",
            name, proc.pid, proc_bar, proc.cpu_usage, proc.memory_mb as u64
        )
    };

    // Left column - first 5 processes
    let left_count = (inner.height as usize).min(5);
    for (i, proc) in status.processes.iter().take(left_count).enumerate() {
        let proc_text = format_process(proc);
        let proc_para = Paragraph::new(proc_text).style(Styles::secondary());
        let proc_area = Rect {
            x: columns[0].x,
            y: columns[0].y + i as u16,
            width: columns[0].width,
            height: 1,
        };
        f.render_widget(proc_para, proc_area);
    }

    // Right column - next 5 processes (6-10)
    let right_count = (inner.height as usize).min(5);
    for (i, proc) in status
        .processes
        .iter()
        .skip(5)
        .take(right_count)
        .enumerate()
    {
        let proc_text = format_process(proc);
        let proc_para = Paragraph::new(proc_text).style(Styles::secondary());
        let proc_area = Rect {
            x: columns[1].x,
            y: columns[1].y + i as u16,
            width: columns[1].width,
            height: 1,
        };
        f.render_widget(proc_para, proc_area);
    }
}

fn create_mini_bar(value: f32, width: usize) -> String {
    let filled = (value.clamp(0.0, 1.0) * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "▰".repeat(filled), "▱".repeat(empty))
}

fn create_progress_bar(value: f32, width: usize) -> String {
    let filled = (value.clamp(0.0, 1.0) * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    // Use block characters for clear progress indication
    format!("{}{}", "▰".repeat(filled), "▱".repeat(empty))
}

fn create_speed_bar(value: f64, width: usize) -> String {
    let filled = (value.clamp(0.0, 1.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "▰".repeat(filled), "▱".repeat(empty))
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

fn format_time(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
