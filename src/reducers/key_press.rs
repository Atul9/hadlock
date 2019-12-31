use {
    crate::{
        config::CONFIG,
        models::{rect::*, window_type::WindowType, windowwrapper::*, Direction, WindowState},
        state::State,
        wm,
        xlibwrapper::action,
        xlibwrapper::core::*,
        xlibwrapper::masks::*,
        xlibwrapper::util::*,
        xlibwrapper::xlibmodels::*,
    },
    reducer::*,
    std::cell::RefCell,
    std::process::Command,
    std::rc::Rc,
};

impl Reducer<action::KeyPress> for State {
    fn reduce(&mut self, action: action::KeyPress) {
        let mod_not_shift = (action.state & (Mod4Mask | Shift)) == Mod4Mask;
        let mod_and_shift = (action.state & (Mod4Mask | Shift)) == Mod4Mask | Shift;

        let ws_keys: Vec<u8> = (1..=9)
            .map(|x| {
                self.lib
                    .str_to_keycode(&x.to_string())
                    .expect("key_press 1")
            })
            .collect();

        let handled_windows = self.windows.keys().map(|key| *key).collect::<Vec<u64>>();
        /*debug!(
            "KeyPress - root: {}, window: {}, handled_windows: {:?}",
            self.lib.get_root(),
            action.win,
            handled_windows
        );*/

        let mon = self
            .monitors
            .get_mut(&self.current_monitor)
            .expect("KeyPress - monitor - get_mut");

        match mon.get_client(self.focus_w) {
            Some(_) => {
                managed_client(self, action, mod_not_shift, mod_and_shift, ws_keys);
            }
            None if action.win == self.lib.get_root() => {
                root(self, action, mod_not_shift, mod_and_shift, ws_keys);
            }
            None => {
                return;
            }
        }
    }
}

fn managed_client(
    state: &mut State,
    action: action::KeyPress,
    mod_not_shift: bool,
    mod_and_shift: bool,
    ws_keys: Vec<u8>,
) {
    debug!("Windows exists: KeyPress");
    let keycode = action.keycode as u8;

    if mod_not_shift && state.lib.str_to_keycode("Return").expect("key_press: 2") == keycode {
        spawn_process(CONFIG.term.as_str(), vec![]);
    }

    if mod_and_shift {
        let old_size = state
            .monitors
            .get(&state.current_monitor)
            .unwrap()
            .get_client(state.focus_w)
            .unwrap()
            .get_size();
        if state.lib.str_to_keycode("Right").expect("key_press: 3") == keycode {
            let mon = state
                .monitors
                .get_mut(&state.current_monitor)
                .expect("KeyPress - monitor - get_mut");
            let (_dec_size, size) =
                mon.resize_window(state.focus_w, old_size.width + 10, old_size.height);
            let ww = mon.remove_window(state.focus_w);
            let new_ww = WindowWrapper {
                window_rect: Rect::new(ww.get_position(), size),
                handle_state: HandleState::Resize.into(),
                ..ww
            };
            mon.add_window(state.focus_w, new_ww);

            return;
        }
        if state.lib.str_to_keycode("Left").expect("key_press: 4") == keycode {
            let mon = state
                .monitors
                .get_mut(&state.current_monitor)
                .expect("KeyPress - monitor - get_mut");
            let (_dec_size, size) =
                mon.resize_window(state.focus_w, old_size.width - 10, old_size.height);
            let ww = mon.remove_window(state.focus_w);
            let new_ww = WindowWrapper {
                window_rect: Rect::new(ww.get_position(), size),
                handle_state: HandleState::Resize.into(),
                ..ww
            };
            mon.add_window(state.focus_w, new_ww);
            return;
        }
        if state.lib.str_to_keycode("Down").expect("key_press: 5") == keycode {
            let mon = state
                .monitors
                .get_mut(&state.current_monitor)
                .expect("KeyPress - monitor - get_mut");
            let (_dec_size, size) =
                mon.resize_window(state.focus_w, old_size.width, old_size.height + 10);
            let ww = mon.remove_window(state.focus_w);
            let new_ww = WindowWrapper {
                window_rect: Rect::new(ww.get_position(), size),
                handle_state: HandleState::Resize.into(),
                ..ww
            };
            mon.add_window(state.focus_w, new_ww);
            return;
        }
        if state.lib.str_to_keycode("Up").expect("key_press: 6") == keycode {
            let mon = state
                .monitors
                .get_mut(&state.current_monitor)
                .expect("KeyPress - monitor - get_mut");
            let (_dec_size, size) =
                mon.resize_window(state.focus_w, old_size.width, old_size.height - 10);
            let ww = mon.remove_window(state.focus_w);
            let new_ww = WindowWrapper {
                window_rect: Rect::new(ww.get_position(), size),
                handle_state: HandleState::Resize.into(),
                ..ww
            };
            mon.add_window(state.focus_w, new_ww);
            return;
        }
        if state.lib.str_to_keycode("q").expect("key_press: 7") == keycode {
            let ww = state
                .monitors
                .get_mut(&state.current_monitor)
                .expect("KeyPress - monitor - get_mut")
                .get_client_mut(state.focus_w)
                .unwrap();
            ww.handle_state.replace(HandleState::Destroy);
            return;
        }
        if state.lib.str_to_keycode("e").expect("key_press: 8") == keycode {
            state.lib.exit();
            return;
        }
        if state.lib.str_to_keycode("f").expect("key_press: 8") == keycode {
            //wm.toggle_monocle(w);
        }

        match ws_keys.contains(&keycode) {
            true => {
                let ws_num = keycode_to_ws(keycode);
                //wm.move_to_ws(w, ws_num as u8);
                wm::set_current_ws(state, ws_num);
            }
            _ => {}
        }
    }

    if mod_not_shift {
        println!("Number pressed");

        if state.lib.str_to_keycode("f").expect("Dafuq?!?!") == keycode {
            let ww = state
                .monitors
                .get_mut(&state.current_monitor)
                .expect("KeyPress - maximize - remove_window")
                .remove_window(state.focus_w);
            let new_ww = wm::toggle_maximize(state, ww);
            state
                .monitors
                .get_mut(&state.current_monitor)
                .expect("KeyPress - maximize - add_window")
                .add_window(state.focus_w, new_ww);
            return;
        }
        if state.lib.str_to_keycode("Right").expect("key_press: 9") == keycode
            || state.lib.str_to_keycode("l").expect("key_press: 10") == keycode
        {
            shift_window(state, Direction::East);
            return;
        }
        if state.lib.str_to_keycode("Left").expect("key_press: 11") == keycode
            || state.lib.str_to_keycode("h").expect("key_press: 12") == keycode
        {
            shift_window(state, Direction::West);
            return;
        }
        if state.lib.str_to_keycode("Down").expect("key_press: 13") == keycode
            || state.lib.str_to_keycode("j").expect("key_press: 14") == keycode
        {
            shift_window(state, Direction::South);
            return;
        }
        if state.lib.str_to_keycode("Up").expect("key_press: \"Up\"") == keycode
            || state.lib.str_to_keycode("k").expect("key_press: 16") == keycode
        {
            debug!("Snap up");
            shift_window(state, Direction::North);
            return;
        }
        if state.lib.str_to_keycode("c").expect("key_press: \"c\"") == keycode {
            debug!("Center window");
            //wm.place_window(wm.focus_w);
            //wm.center_cursor(wm.focus_w);
            return;
        }
        if state.lib.str_to_keycode("d").expect("key_press: \"d\"") == keycode {
            debug!("dmenu_run");
            spawn_process("dmenu_recency", vec![]);
            return;
        }
        if ws_keys.contains(&keycode) {
            debug!("mod_not_shift switch ws");
            let ws_num = keycode_to_ws(keycode);
            wm::set_current_ws(state, ws_num);
        }
    }
}

fn root(
    state: &mut State,
    action: action::KeyPress,
    mod_not_shift: bool,
    mod_and_shift: bool,
    ws_keys: Vec<u8>,
) {
    let keycode = action.keycode as u8;
    if mod_not_shift {
        if state.lib.str_to_keycode("Return").expect("key_press: 17") == keycode {
            spawn_process(CONFIG.term.as_str(), vec![]);
        }
        if state.lib.str_to_keycode("d").expect("key_press: \"d\"") == keycode {
            debug!("dmenu_run");
            spawn_process("dmenu_recency", vec![]);
            return;
        }

        match ws_keys.contains(&keycode) {
            true => {
                let ws_num = keycode_to_ws(keycode);
                wm::set_current_ws(state, ws_num);
            }
            _ => {}
        }
    }
    if mod_and_shift {
        if state.lib.str_to_keycode("e").expect("key_press: 18") == keycode {
            state.lib.exit();
        }
    }
}

fn shift_window(state: &mut State, direction: Direction) {
    let mon = state
        .monitors
        .get_mut(&state.current_monitor)
        .expect("KeyPress - shift_window - monitor - get_mut");
    let (pos, size) = mon.shift_window(state.focus_w, direction);
    let ww = mon.remove_window(state.focus_w);
    let ww = WindowWrapper {
        window_rect: Rect::new(pos, size),
        current_state: WindowState::Snapped,
        handle_state: HandleState::Shift.into(),
        ..ww
    };
    mon.add_window(state.focus_w, ww);
}

fn keycode_to_ws(keycode: u8) -> u32 {
    ((keycode - 10) % 10) as u32
}

fn spawn_process(bin_name: &str, args: Vec<&str>) {
    let mut cmd = Command::new(bin_name);
    args.into_iter().for_each(|arg| {
        cmd.arg(arg);
    });
    let _ = cmd.spawn();
}