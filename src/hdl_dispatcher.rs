use {
    crate::hdl_reactor::HdlReactor,
    crate::state::State,
    crate::xlibwrapper::core::XlibWrapper,
    crate::xlibwrapper::{action, xlibmodels::*},
    reducer::*,
    std::rc::Rc,
    x11_dl::xlib,
    std::sync::mpsc::*,
};

pub fn run(xlib: Rc<XlibWrapper>, sender: Sender<bool>) {



    let (tx, rx) = channel::<()>();
    let state = State::new(xlib.clone());
    let mut store = Store::new(state, HdlReactor::new(xlib.clone(), tx));

    //setup 
    xlib.grab_server();
    let _ = xlib.get_top_level_windows()
        .iter()
        .map(|w| {
            store.dispatch(action::MapRequest{win: *w, parent: xlib.get_root()});
        });
    xlib.ungrab_server();
    let _ = sender.send(true);

    loop {
        let xevent = xlib.next_event();
        match xevent.get_type() {
            xlib::ConfigureRequest => {
                let event = xlib::XConfigureRequestEvent::from(xevent);
                let window_changes = WindowChanges {
                    x: event.x,
                    y: event.y,
                    width: event.width,
                    height: event.height,
                    border_width: event.border_width,
                    sibling: event.above,
                    stack_mode: event.detail
                };
                store.dispatch(action::ConfigurationRequest{win: event.window, win_changes: window_changes, value_mask: event.value_mask, parent: event.parent})
            },
            xlib::MapRequest => {
                let event = xlib::XMapRequestEvent::from(xevent);
                store.dispatch(action::MapRequest { win: event.window, parent: event.parent })
            }
            xlib::UnmapNotify => {
                let event = xlib::XUnmapEvent::from(xevent);
                store.dispatch(action::UnmapNotify { win: event.window })
            }
            xlib::ButtonPress => {
                let event = xlib::XButtonEvent::from(xevent);
                store.dispatch(action::ButtonPress {
                    win: event.window,
                    sub_win: event.subwindow,
                    button: event.button,
                    x_root: event.x_root as u32,
                    y_root: event.y_root as u32,
                    state: event.state as u32,
                });
            }
            xlib::ButtonRelease => {
                let event = xlib::XButtonEvent::from(xevent);
                store.dispatch(action::ButtonRelease {
                    win: event.window,
                    sub_win: event.subwindow,
                    button: event.button,
                    x_root: event.x_root as u32,
                    y_root: event.y_root as u32,
                    state: event.state as u32,
                })
            }
            xlib::KeyPress => {
                let event = xlib::XKeyEvent::from(xevent);
                store.dispatch(action::KeyPress {
                    win: event.window,
                    state: event.state,
                    keycode: event.keycode,
                })
            }
            /*xlib::KeyRelease => {
              let event = xlib::XKeyEvent::from(xevent);
              action::KeyRelease{win: event.window, state: event.state, keycode: event.keycode};
              },*/
            xlib::MotionNotify => {
                let event = xlib::XMotionEvent::from(xevent);
                store.dispatch(action::MotionNotify {
                    win: event.window,
                    sub_win: event.subwindow,
                    x_root: event.x_root,
                    y_root: event.y_root,
                    state: event.state,
                })
            }
            xlib::EnterNotify => {
                let event = xlib::XCrossingEvent::from(xevent);
                store.dispatch(action::EnterNotify {
                    win: event.window,
                    sub_win: event.subwindow,
                })
            }
            xlib::LeaveNotify => {
                let event = xlib::XCrossingEvent::from(xevent);
                store.dispatch(action::LeaveNotify { win: event.window })
            }
            /*xlib::Expose => {
              let event = xlib::XExposeEvent::from(xevent);
              action::Expose{win: event.window};
              },*/
            xlib::DestroyNotify => {
                let event = xlib::XDestroyWindowEvent::from(xevent);
                store.dispatch(action::DestroyNotify { win: event.window })
            }
            xlib::PropertyNotify => {
                let event = xlib::XPropertyEvent::from(xevent);
                action::PropertyNotify {
                    win: event.window,
                    atom: event.atom,
                };
            }
            xlib::ClientMessage => {
                let event = xlib::XClientMessageEvent::from(xevent);
                store.dispatch(action::ClientMessageRequest {
                    win: event.window,
                    message_type: event.message_type,
                    data: vec![
                        event.data.get_long(0),
                        event.data.get_long(1),
                        event.data.get_long(2),
                    ],
                });
            }
            _ => store.dispatch(action::UnknownEvent)
        }

        match rx.try_recv() {
            Ok(_) => {
                debug!("Motion dispatch focus");
                if let Some(w) = xlib.window_under_pointer() {
                    store.dispatch(action::Focus{win: w})
                }
            },
            Err(_) => ()
        }
    }
}
