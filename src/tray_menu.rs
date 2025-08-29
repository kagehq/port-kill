use crate::types::{ProcessInfo, StatusBarInfo};
use anyhow::Result;
use crossbeam_channel::Sender;
use log::debug;
use std::collections::HashMap;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon,
};

#[derive(Clone)]
pub struct TrayMenu {
    pub menu: Menu,
    pub icon: Icon,
    menu_sender: Sender<MenuEvent>,
}

impl TrayMenu {
    pub fn new(menu_sender: Sender<MenuEvent>) -> Result<Self> {
        // Create a simple icon (we'll use a text-based approach for now)
        let icon = Self::create_icon("0")?;

        // Create initial menu
        let menu = Self::create_menu(&HashMap::new(), false)?;

        // Set up menu event handling
        let sender_clone = menu_sender.clone();
        MenuEvent::set_event_handler(Some(move |event| {
            let _ = sender_clone.send(event);
        }));

        Ok(Self {
            menu,
            icon,
            menu_sender,
        })
    }

    pub fn update_menu(&mut self, processes: &HashMap<u16, ProcessInfo>) -> Result<()> {
        debug!("Updating menu with {} processes", processes.len());
        
        // Create new menu with current processes
        let new_menu = Self::create_menu(processes, false)?;
        self.menu = new_menu;
        
        Ok(())
    }

    pub fn update_status(&mut self, status_info: &StatusBarInfo) -> Result<()> {
        debug!("Updating status bar: {}", status_info.text);
        
        // Update icon with new status text
        self.icon = Self::create_icon(&status_info.text)?;
        
        Ok(())
    }

    pub fn create_menu(processes: &HashMap<u16, ProcessInfo>, show_pid: bool) -> Result<Menu> {
        let menu = Menu::new();

        // Add "Kill All Processes" item with proper ID
        let kill_all_item = MenuItem::new("Kill All Processes", true, None);
        menu.append(&kill_all_item)?;

        // Add separator
        let separator = PredefinedMenuItem::separator();
        menu.append(&separator)?;

        // Add individual process items with proper IDs
        for (port, process_info) in processes {
            let menu_text = if let (Some(_container_id), Some(container_name)) = (&process_info.container_id, &process_info.container_name) {
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
                format!(
                    "Kill: Port {}: {}",
                    port, process_info.name
                )
            };
            
            // Create unique menu ID for each process
            let menu_id = format!("process_{}_{}", port, process_info.pid);
            
            // For now, we'll use the menu text to identify processes since tray-icon doesn't support custom IDs
            // The menu event will contain the menu text which we can parse
            let process_item = MenuItem::new(&menu_text, true, None);
            menu.append(&process_item)?;
        }

        // Add another separator if there are processes
        if !processes.is_empty() {
            let separator = PredefinedMenuItem::separator();
            menu.append(&separator)?;
        }

        // Add "Quit" item with proper ID
        let quit_item = MenuItem::new("Quit", true, None);
        menu.append(&quit_item)?;

        Ok(menu)
    }



    pub fn create_icon(text: &str) -> Result<Icon> {
        // Create a simple but visible icon for the status bar
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
                        // For single digits, make the number area more prominent
                        (255, 0, 0, 255) // Bright red background for number
                    } else {
                        // For double digits, use orange to indicate many processes
                        (255, 165, 0, 255) // Orange for 10+ processes
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
