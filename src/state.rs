use {
    std::rc::Rc,
    std::collections::HashMap,
    crate::models::{
        monitor::Monitor,
        workspace::Workspace,
        windowwrapper::WindowWrapper,
        screen::Screen,
    },
    crate::xlibwrapper::{
        masks::*,
        xlibmodels::*,
        core::*,
        xatom::*,
    },
    derivative::*,
    std::cell::RefCell,
};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct State {
    #[derivative(Debug="ignore")]
    pub lib: Rc<XlibWrapper>,
    pub windows: HashMap<Window, WindowWrapper>,
    pub focus_w: Window,
    pub monitors: HashMap<MonitorId, Monitor>,
    pub current_monitor: MonitorId,
    pub drag_start_pos: (i32, i32),
    pub drag_start_frame_pos: (i32, i32),
    pub drag_start_frame_size: (u32, u32),
}

impl State {
    pub fn new(lib: Rc<XlibWrapper>) -> Self {
        let focus_w = lib.get_root();
        let monitors = {
            let mut monitors = HashMap::default();
            let _ = lib.get_screens()
                .iter()
                .enumerate()
                .for_each(|(i, val)| {
                    info!("Monitors in init: {}", i);
                    monitors.insert(i as u32, Monitor::new(i as u32, val.clone(), Workspace::new(i as u32)));
                });
            let mon_count = monitors.iter().count();
            debug!("Monitor on start: {}", mon_count);
            monitors
        };
        Self {
            lib,
            windows: HashMap::default(),
            focus_w,
            monitors,
            current_monitor: 0,
            drag_start_pos: (0,0),
            drag_start_frame_pos: (0,0),
            drag_start_frame_size: (0,0),
        }
    }

    pub fn pointer_is_inside(&self, screen: &Screen) -> bool {
        let pointer_pos = self.lib.pointer_pos();
        //debug!("pointer pos: {:?}", pointer_pos);
        let inside_height = pointer_pos.y >= screen.y &&
            pointer_pos.y <= screen.y + screen.height as i32;

        let inside_width = pointer_pos.x >= screen.x &&
            pointer_pos.x <= screen.x + screen.width as i32;

        inside_height && inside_width
    }
    pub fn get_monitor_by_mouse(&self) -> MonitorId {
        let mon_vec = self.monitors
            .iter()
            .filter(|(_key, mon)| self.pointer_is_inside(&mon.screen))
            .map(|(key, _mon)| *key)
            .collect::<Vec<u32>>();
        match mon_vec.get(0) {
            Some(mon_id) => *mon_id,
            None => self.current_monitor
        }
    }
}