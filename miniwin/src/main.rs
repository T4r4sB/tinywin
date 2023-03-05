use core::mem::MaybeUninit;

use winapi::shared::minwindef::{LPARAM, LPVOID, LRESULT, UINT, WPARAM};

use winapi::shared::windef::{HBRUSH, HICON, HMENU, HWND, RECT};

use winapi::um::libloaderapi::GetModuleHandleA;

use winapi::um::winuser::{
    BeginPaint, CreateWindowExA, DefWindowProcA, DispatchMessageA, DrawTextA, EndPaint,
    GetClientRect, GetMessageA, GetWindowRect, PostQuitMessage, RegisterClassA,
    SystemParametersInfoA, TranslateMessage,
};

use winapi::um::winuser::{
    CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CW_USEDEFAULT, DT_CENTER, DT_SINGLELINE, DT_VCENTER,
    SPI_GETWORKAREA, WNDCLASSA, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

use winapi::um::dwmapi::DwmGetWindowAttribute;
use winapi::um::dwmapi::DWMWA_EXTENDED_FRAME_BOUNDS;

pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        winapi::um::winuser::WM_PAINT => {
            let mut paint_struct = MaybeUninit::uninit();
            let mut rect = MaybeUninit::uninit();

            let hdc = BeginPaint(hwnd, paint_struct.as_mut_ptr());
            GetClientRect(hwnd, rect.as_mut_ptr());
            DrawTextA(
                hdc,
                "Hello world\0".as_ptr() as *const i8,
                -1,
                rect.as_mut_ptr(),
                DT_SINGLELINE | DT_CENTER | DT_VCENTER,
            );

            let ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let x = (ms & 255) as usize;
            winapi::um::wingdi::SetPixel(hdc, x as i32, x as i32, ms as u32);
            EndPaint(hwnd, paint_struct.as_mut_ptr());
        }
        winapi::um::winuser::WM_TIMER => {
            let mut rect = MaybeUninit::uninit();
            GetWindowRect(hwnd, rect.as_mut_ptr());
            let window_rect = rect.assume_init();
            let mut frame_rect = window_rect;

            let dwa_result = DwmGetWindowAttribute(
                hwnd,
                DWMWA_EXTENDED_FRAME_BOUNDS,
                &mut frame_rect as *mut RECT as *mut _,
                std::mem::size_of::<RECT>() as u32,
            );
            // dont care about result, window_rect is ok for us

            let mut rect = MaybeUninit::<RECT>::uninit();
            SystemParametersInfoA(
                SPI_GETWORKAREA,
                0,
                std::mem::transmute(rect.as_mut_ptr()),
                0,
            );
            let desktop_rect = rect.assume_init();

            println!("dwa_result = {:x}", dwa_result);

            println!(
                "window_rect={}:{}-{}:{}",
                window_rect.left, window_rect.top, window_rect.right, window_rect.bottom
            );

            println!(
                "frame_rect={}:{}-{}:{}",
                frame_rect.left, frame_rect.top, frame_rect.right, frame_rect.bottom
            );

            println!(
                "desktop_rect={}:{}-{}:{}",
                desktop_rect.left, desktop_rect.top, desktop_rect.right, desktop_rect.bottom
            );

            winapi::um::winuser::InvalidateRect(hwnd, 0 as *const winapi::shared::windef::RECT, 0);
        }
        winapi::um::winuser::WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => {
            return DefWindowProcA(hwnd, msg, wparam, lparam);
        }
    }
    return 0;
}

fn create_window() -> HWND {
    unsafe {
        let hinstance = GetModuleHandleA(0 as *const i8);
        let wnd_class = WNDCLASSA {
            style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: hinstance,
            lpszClassName: "MyClass\0".as_ptr() as *const i8,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: 0 as HICON,
            hCursor: 0 as HICON,
            hbrBackground: 0 as HBRUSH,
            lpszMenuName: 0 as *const i8,
        };
        RegisterClassA(&wnd_class);

        let hwnd = CreateWindowExA(
            0,                                 // dwExStyle
            "MyClass\0".as_ptr() as *const i8, // class we registered.
            "MiniWIN\0".as_ptr() as *const i8, // title
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,  // dwStyle
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT, // size and position
            0 as HWND,     // hWndParent
            0 as HMENU,    // hMenu
            hinstance,     // hInstance
            0 as LPVOID,
        ); // lpParam

        winapi::um::winuser::SetTimer(hwnd, 0, 100, None);
        hwnd
    }
}

// More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms644927(v=vs.85).aspx
fn handle_message(window: HWND) -> bool {
    unsafe {
        let mut msg = MaybeUninit::uninit();
        if GetMessageA(msg.as_mut_ptr(), window, 0, 0) > 0 {
            TranslateMessage(msg.as_ptr());
            DispatchMessageA(msg.as_ptr());
            true
        } else {
            false
        }
    }
}

pub fn main() {
    let window = create_window();
    loop {
        if !handle_message(window) {
            break;
        }
    }
}
