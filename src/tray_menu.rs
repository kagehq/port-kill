use crate::types::{ProcessInfo, StatusBarInfo};
use anyhow::Result;
use crossbeam_channel::Sender;
use log::debug;
use std::collections::HashMap;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use tray_icon::{
    menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem},
    Icon,
};
#[cfg(target_os = "windows")]
use tray_icon::menu::Submenu;

#[cfg(any(target_os = "macos", target_os = "windows"))]
#[derive(Clone)]
pub struct TrayMenu {
    pub icon: Icon,
    current_processes: HashMap<u16, ProcessInfo>,
    show_pid: bool,
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
impl TrayMenu {
    pub fn new(_menu_sender: Sender<MenuEvent>) -> Result<Self> {
        // Create a simple icon (we'll use a text-based approach for now)
        let icon = Self::create_icon("0")?;

        // Set up menu event handling
        let sender_clone = _menu_sender.clone();
        MenuEvent::set_event_handler(Some(move |event| {
            let _ = sender_clone.send(event);
        }));

        Ok(Self {
            icon,
            current_processes: HashMap::new(),
            show_pid: false,
        })
    }

    pub fn update_menu(
        &mut self,
        processes: &HashMap<u16, ProcessInfo>,
        show_pid: bool,
    ) -> Result<()> {
        debug!("Updating menu with {} processes", processes.len());

        // Update internal state
        self.current_processes = processes.clone();
        self.show_pid = show_pid;

        Ok(())
    }

    pub fn get_current_menu(&self) -> Result<Menu> {
        let (menu, _) = Self::create_menu(&self.current_processes, self.show_pid)?;
        Ok(menu)
    }

    pub fn update_status(&mut self, status_info: &StatusBarInfo) -> Result<()> {
        debug!("Updating status bar: {}", status_info.text);

        // Update icon with new status text
        self.icon = Self::create_icon(&status_info.text)?;

        Ok(())
    }

    pub fn create_menu(
        processes: &HashMap<u16, ProcessInfo>,
        show_pid: bool,
    ) -> Result<(Menu, HashMap<String, u16>)> {
        Self::create_menu_with_verbose(processes, show_pid, false)
    }

    pub fn create_menu_with_verbose(
        processes: &HashMap<u16, ProcessInfo>,
        show_pid: bool,
        verbose: bool,
    ) -> Result<(Menu, HashMap<String, u16>)> {
        let menu = Menu::new();
        let mut menu_id_to_port = HashMap::new();

        // Add "Kill All Processes" item
        let kill_all_item = MenuItem::with_id(MenuId("kill_all".into()), "Kill All Processes", true, None);
        menu.append(&kill_all_item)?;

        // Add separator
        let separator = PredefinedMenuItem::separator();
        menu.append(&separator)?;

        // Windows: group processes by type into submenus ("Docker (7)", ...) and
        // keep everything sorted by port. Groups with a single member and
        // ungrouped processes stay at the top level.
        #[cfg(target_os = "windows")]
        {
            use std::collections::BTreeMap;

            let mut groups: BTreeMap<String, Vec<(u16, &ProcessInfo)>> = BTreeMap::new();
            let mut top_level: Vec<(u16, &ProcessInfo)> = Vec::new();

            let mut sorted: Vec<(&u16, &ProcessInfo)> = processes.iter().collect();
            sorted.sort_by_key(|(port, _)| **port);

            for (port, process_info) in sorted {
                match &process_info.process_group {
                    Some(group) => groups.entry(group.clone()).or_default().push((*port, process_info)),
                    None => top_level.push((*port, process_info)),
                }
            }

            for (group_name, members) in &groups {
                if members.len() >= 2 {
                    let submenu = Submenu::new(format!("{} ({})", group_name, members.len()), true);
                    for (port, process_info) in members {
                        let menu_text =
                            Self::format_process_menu_text(*port, process_info, show_pid, verbose);
                        let process_item = MenuItem::new(&menu_text, true, None);
                        menu_id_to_port.insert(process_item.id().0.clone(), *port);
                        submenu.append(&process_item)?;
                    }
                    menu.append(&submenu)?;
                } else {
                    top_level.extend(members.iter().map(|(port, info)| (*port, *info)));
                }
            }

            top_level.sort_by_key(|(port, _)| *port);
            for (port, process_info) in top_level {
                let menu_text = Self::format_process_menu_text(port, process_info, show_pid, verbose);
                let process_item = MenuItem::new(&menu_text, true, None);
                menu_id_to_port.insert(process_item.id().0.clone(), port);
                menu.append(&process_item)?;
            }
        }

        // macOS: original flat list
        #[cfg(not(target_os = "windows"))]
        for (port, process_info) in processes {
            let menu_text = Self::format_process_menu_text(*port, process_info, show_pid, verbose);

            // Create menu item for each process
            let process_item = MenuItem::new(&menu_text, true, None);
            let process_id = process_item.id();
            menu.append(&process_item)?;

            // Store the mapping from menu ID to port
            menu_id_to_port.insert(process_id.0.clone(), *port);
        }

        // Add another separator if there are processes
        if !processes.is_empty() {
            let separator = PredefinedMenuItem::separator();
            menu.append(&separator)?;
        }

        // Windows: "Start with Windows" toggle backed by the HKCU Run key
        #[cfg(target_os = "windows")]
        {
            use tray_icon::menu::CheckMenuItem;
            let autostart_item = CheckMenuItem::with_id(
                MenuId("toggle_autostart".into()),
                "Start with Windows",
                true,
                crate::autostart::is_enabled(),
                None,
            );
            menu.append(&autostart_item)?;
            let separator = PredefinedMenuItem::separator();
            menu.append(&separator)?;
        }

        // Add "Quit" item
        let quit_item = MenuItem::with_id(MenuId("quit".into()), "Quit", true, None);
        menu.append(&quit_item)?;

        Ok((menu, menu_id_to_port))
    }

    /// Build the label for a process menu entry (shared by the flat macOS
    /// list and the grouped Windows menu).
    fn format_process_menu_text(
        port: u16,
        process_info: &ProcessInfo,
        show_pid: bool,
        verbose: bool,
    ) -> String {
        if verbose {
            // Verbose mode: show command line and working directory
            let mut parts = vec![format!("Kill: Port {}: {}", port, process_info.name)];

            if let Some(ref cmd_line) = process_info.command_line {
                parts.push(format!("({})", cmd_line));
            }

            if show_pid {
                parts.push(format!("(PID {})", process_info.pid));
            }

            if let Some(ref work_dir) = process_info.working_directory {
                parts.push(format!("- {}", Self::short_display_path(work_dir)));
            }

            if let (Some(_container_id), Some(container_name)) =
                (&process_info.container_id, &process_info.container_name)
            {
                parts.push(format!("[Docker: {}]", container_name));
            }

            parts.join(" ")
        } else if let (Some(_container_id), Some(container_name)) =
            (&process_info.container_id, &process_info.container_name)
        {
            format!(
                "Kill: Port {}: {} [Docker: {}]",
                port, process_info.name, container_name
            )
        } else if show_pid {
            format!(
                "Kill: Port {}: {} (PID {})",
                port, process_info.name, process_info.pid
            )
        } else {
            format!("Kill: Port {}: {}", port, process_info.name)
        }
    }

    fn short_display_path(dir: &str) -> String {
        let parts: Vec<&str> = dir.split(['/', '\\']).collect();
        if parts.len() >= 2 {
            parts[parts.len() - 2..].join("/")
        } else {
            dir.to_string()
        }
    }

    pub fn create_icon(text: &str) -> Result<Icon> {
        // Windows: colored badge with the process count drawn on it
        #[cfg(target_os = "windows")]
        {
            let icon_data = Self::generate_count_icon(text);
            return Icon::from_rgba(icon_data, 32, 32)
                .map_err(|e| anyhow::anyhow!("Failed to create icon: {}", e));
        }

        // macOS: original white square with colored center
        #[cfg(not(target_os = "windows"))]
        {
            let icon_data = Self::generate_visible_icon(text);

            // Try different sizes for better compatibility
            match Icon::from_rgba(icon_data.clone(), 16, 16) {
                Ok(icon) => Ok(icon),
                Err(_) => {
                    // Fallback to 32x32
                    Icon::from_rgba(icon_data, 32, 32)
                        .map_err(|e| anyhow::anyhow!("Failed to create icon: {}", e))
                }
            }
        }
    }

    /// Windows: 32x32 RGBA badge whose color follows the process count
    /// (green = 0, orange = 1-9, red = 10+) with the count drawn in white.
    #[cfg(target_os = "windows")]
    fn generate_count_icon(text: &str) -> Vec<u8> {
        const SIZE: usize = 32;
        const CORNER_RADIUS: i32 = 8;
        // 3x5 bitmap font for digits 0-9 (one u8 per row, 3 lowest bits used)
        const FONT: [[u8; 5]; 10] = [
            [0b111, 0b101, 0b101, 0b101, 0b111], // 0
            [0b010, 0b110, 0b010, 0b010, 0b111], // 1
            [0b111, 0b001, 0b111, 0b100, 0b111], // 2
            [0b111, 0b001, 0b111, 0b001, 0b111], // 3
            [0b101, 0b101, 0b111, 0b001, 0b001], // 4
            [0b111, 0b100, 0b111, 0b001, 0b111], // 5
            [0b111, 0b100, 0b111, 0b101, 0b111], // 6
            [0b111, 0b001, 0b010, 0b010, 0b010], // 7
            [0b111, 0b101, 0b111, 0b101, 0b111], // 8
            [0b111, 0b101, 0b111, 0b001, 0b111], // 9
        ];

        let count: u32 = text
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .unwrap_or(0);
        let display = if count > 99 {
            "99".to_string()
        } else {
            count.to_string()
        };

        let (r, g, b) = if count == 0 {
            (0x2e, 0xcc, 0x71) // green
        } else if count <= 9 {
            (0xf3, 0x9c, 0x12) // orange
        } else {
            (0xe7, 0x4c, 0x3c) // red
        };

        // Rounded-square badge over a transparent background
        let mut data = vec![0u8; SIZE * SIZE * 4];
        for y in 0..SIZE {
            for x in 0..SIZE {
                let xi = x as i32;
                let yi = y as i32;
                let max = SIZE as i32 - 1;
                // Distance check against the four rounded corners
                let corner_center = |cx: i32, cy: i32| {
                    let dx = xi - cx;
                    let dy = yi - cy;
                    dx * dx + dy * dy > CORNER_RADIUS * CORNER_RADIUS
                };
                let in_corner = (xi < CORNER_RADIUS && yi < CORNER_RADIUS && corner_center(CORNER_RADIUS, CORNER_RADIUS))
                    || (xi > max - CORNER_RADIUS && yi < CORNER_RADIUS && corner_center(max - CORNER_RADIUS, CORNER_RADIUS))
                    || (xi < CORNER_RADIUS && yi > max - CORNER_RADIUS && corner_center(CORNER_RADIUS, max - CORNER_RADIUS))
                    || (xi > max - CORNER_RADIUS
                        && yi > max - CORNER_RADIUS
                        && corner_center(max - CORNER_RADIUS, max - CORNER_RADIUS));
                if !in_corner {
                    let idx = (y * SIZE + x) * 4;
                    data[idx..idx + 4].copy_from_slice(&[r, g, b, 255]);
                }
            }
        }

        // Draw the count centered, scaled up from the 3x5 font
        let scale = if display.len() == 1 { 4 } else { 3 };
        let glyph_w = 3 * scale;
        let glyph_h = 5 * scale;
        let gap = scale;
        let total_w = display.len() * glyph_w + (display.len() - 1) * gap;
        let x0 = (SIZE - total_w) / 2;
        let y0 = (SIZE - glyph_h) / 2;
        for (i, ch) in display.chars().enumerate() {
            let digit = ch.to_digit(10).unwrap_or(0) as usize;
            let gx = x0 + i * (glyph_w + gap);
            for row in 0..5 {
                for col in 0..3 {
                    if (FONT[digit][row] >> (2 - col)) & 1 == 1 {
                        for dy in 0..scale {
                            for dx in 0..scale {
                                let px = gx + col * scale + dx;
                                let py = y0 + row * scale + dy;
                                let idx = (py * SIZE + px) * 4;
                                data[idx..idx + 4].copy_from_slice(&[255, 255, 255, 255]);
                            }
                        }
                    }
                }
            }
        }

        data
    }

    #[cfg(not(target_os = "windows"))]
    fn generate_visible_icon(text: &str) -> Vec<u8> {
        // Create a much larger, highly visible 32x32 RGBA icon for the status bar
        let mut icon_data = Vec::new();

        for y in 0..32 {
            for x in 0..32 {
                // Create a very simple, highly visible icon
                let _is_edge = x < 2 || x > 29 || y < 2 || y > 29;
                let _is_center = x >= 14 && x <= 17 && y >= 14 && y <= 17;

                // Create a number display area in the center
                let is_number_area = x >= 12 && x <= 19 && y >= 12 && y <= 19;

                let (r, g, b, a) = if is_number_area {
                    // Parse the number from text (remove any non-numeric characters)
                    let number = text.chars().filter(|c| c.is_numeric()).collect::<String>();
                    let num = number.parse::<u32>().unwrap_or(0);

                    if num == 0 {
                        (0, 255, 0, 255) // Bright green when no processes
                    } else if num <= 9 {
                        // For 1-9 processes, use orange
                        (255, 165, 0, 255) // Orange for 1-9 processes
                    } else {
                        // For 10+ processes, use red to indicate many processes
                        (255, 0, 0, 255) // Red for 10+ processes
                    }
                } else {
                    (255, 255, 255, 255) // Clean white background
                };

                icon_data.extend_from_slice(&[r, g, b, a]);
            }
        }

        icon_data
    }
}

#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod tests {
    use super::TrayMenu;
    use crate::types::ProcessInfo;

    fn process(port: u16, pid: i32, name: &str) -> ProcessInfo {
        ProcessInfo {
            pid,
            port,
            command: name.to_string(),
            name: name.to_string(),
            container_id: None,
            container_name: None,
            command_line: None,
            working_directory: None,
            process_group: None,
            project_name: None,
            cpu_usage: None,
            memory_usage: None,
            memory_percentage: None,
        }
    }

    #[test]
    fn menu_text_basic() {
        let info = process(3000, 1234, "node.exe");
        assert_eq!(
            TrayMenu::format_process_menu_text(3000, &info, false, false),
            "Kill: Port 3000: node.exe"
        );
    }

    #[test]
    fn menu_text_with_pid() {
        let info = process(3000, 1234, "node.exe");
        assert_eq!(
            TrayMenu::format_process_menu_text(3000, &info, true, false),
            "Kill: Port 3000: node.exe (PID 1234)"
        );
    }

    #[test]
    fn menu_text_docker_takes_precedence() {
        let mut info = process(5432, 42, "postgres");
        info.container_id = Some("abc123".to_string());
        info.container_name = Some("my-db".to_string());
        assert_eq!(
            TrayMenu::format_process_menu_text(5432, &info, true, false),
            "Kill: Port 5432: postgres [Docker: my-db]"
        );
    }

    #[test]
    fn short_display_path_handles_both_separators() {
        assert_eq!(
            TrayMenu::short_display_path("/Users/dev/projects/my-app"),
            "projects/my-app"
        );
        assert_eq!(
            TrayMenu::short_display_path(r"C:\Users\dev\projects\my-app"),
            "projects/my-app"
        );
        assert_eq!(TrayMenu::short_display_path("my-app"), "my-app");
    }

    #[cfg(target_os = "windows")]
    mod windows_icon {
        use super::super::TrayMenu;

        fn center_pixel(data: &[u8]) -> (u8, u8, u8, u8) {
            let idx = (16 * 32 + 16) * 4;
            (data[idx], data[idx + 1], data[idx + 2], data[idx + 3])
        }

        fn corner_pixel(data: &[u8]) -> u8 {
            // Alpha of the top-left pixel (should be transparent due to rounding)
            data[3]
        }

        #[test]
        fn icon_has_expected_dimensions() {
            let data = TrayMenu::generate_count_icon("0");
            assert_eq!(data.len(), 32 * 32 * 4);
        }

        #[test]
        fn icon_color_follows_count() {
            // Green when idle; the exact center is covered by the white digit,
            // so sample an edge pixel of the badge instead
            let sample = |text: &str| {
                let data = TrayMenu::generate_count_icon(text);
                let idx = (16 * 32 + 2) * 4; // left edge, vertically centered
                (data[idx], data[idx + 1], data[idx + 2])
            };
            assert_eq!(sample("0"), (0x2e, 0xcc, 0x71)); // green
            assert_eq!(sample("3"), (0xf3, 0x9c, 0x12)); // orange
            assert_eq!(sample("12"), (0xe7, 0x4c, 0x3c)); // red
        }

        #[test]
        fn icon_corners_are_transparent() {
            let data = TrayMenu::generate_count_icon("5");
            assert_eq!(corner_pixel(&data), 0);
        }

        #[test]
        fn icon_draws_white_digits() {
            // The digit strokes are white; ensure at least one white pixel exists
            let data = TrayMenu::generate_count_icon("8");
            let has_white = data
                .chunks_exact(4)
                .any(|p| p == [255, 255, 255, 255]);
            assert!(has_white);
            let _ = center_pixel(&data);
        }
    }
}
