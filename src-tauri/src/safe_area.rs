use std::f64;

#[derive(Debug, Clone, Copy)]
pub struct SafeAreaInsets {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
}

impl Default for SafeAreaInsets {
    fn default() -> Self {
        Self {
            top: 47.0,
            bottom: 34.0,
            left: 0.0,
            right: 0.0,
        }
    }
}

#[cfg(target_os = "ios")]
mod ios_impl {
    use super::SafeAreaInsets;
    use objc::runtime::*;
    use objc::*;
    
    pub fn get_safe_area_insets() -> SafeAreaInsets {
        unsafe {
            let uiscreen: *mut Object = msg_send![class!(UIScreen), mainScreen];
            let safe_area: *mut Object = msg_send![uiscreen, safeAreaInsets];
            
            let top: f64 = msg_send![safe_area, top];
            let bottom: f64 = msg_send![safe_area, bottom];
            let left: f64 = msg_send![safe_area, left];
            let right: f64 = msg_send![safe_area, right];
            
            SafeAreaInsets { top, bottom, left, right }
        }
    }
    
    pub fn inject_safe_area_to_window(window: &tauri::WebviewWindow, insets: SafeAreaInsets) {
        let script = format!(
            r#"
            (function() {{
                document.documentElement.style.setProperty('--wsf-safe-top', '{}px', 'important');
                document.documentElement.style.setProperty('--wsf-safe-bottom', '{}px', 'important');
                document.documentElement.style.setProperty('--wsf-safe-left', '{}px', 'important');
                document.documentElement.style.setProperty('--wsf-safe-right', '{}px', 'important');
                
                // 设置 body padding
                var body = document.body;
                if (body) {{
                    body.style.setProperty('padding-top', '{}px', 'important');
                    body.style.setProperty('padding-bottom', '{}px', 'important');
                }}
                
                // 设置常见容器
                var containers = document.querySelectorAll('#app, #root, main, .app, .main');
                containers.forEach(function(el) {{
                    el.style.setProperty('padding-top', '{}px', 'important');
                    el.style.setProperty('padding-bottom', '{}px', 'important');
                }});
            }})();
            "#,
            insets.top, insets.bottom, insets.left, insets.right,
            insets.top, insets.bottom,
            insets.top, insets.bottom
        );
        
        let _ = window.eval(&script);
    }
}

#[cfg(target_os = "android")]
mod android_impl {
    use super::SafeAreaInsets;
    
    pub fn get_safe_area_insets() -> SafeAreaInsets {
        // Android 实现略，暂时返回默认值
        SafeAreaInsets::default()
    }
    
    pub fn inject_safe_area_to_window(window: &tauri::WebviewWindow, insets: SafeAreaInsets) {
        // 与 iOS 相同的注入逻辑
        let script = format!(
            r#"
            (function() {{
                document.documentElement.style.setProperty('--wsf-safe-top', '{}px', 'important');
                document.documentElement.style.setProperty('--wsf-safe-bottom', '{}px', 'important');
                document.documentElement.style.setProperty('--wsf-safe-left', '{}px', 'important');
                document.documentElement.style.setProperty('--wsf-safe-right', '{}px', 'important');
                
                var body = document.body;
                if (body) {{
                    body.style.setProperty('padding-top', '{}px', 'important');
                    body.style.setProperty('padding-bottom', '{}px', 'important');
                }}
                
                var containers = document.querySelectorAll('#app, #root, main, .app, .main');
                containers.forEach(function(el) {{
                    el.style.setProperty('padding-top', '{}px', 'important');
                    el.style.setProperty('padding-bottom', '{}px', 'important');
                }});
            }})();
            "#,
            insets.top, insets.bottom, insets.left, insets.right,
            insets.top, insets.bottom,
            insets.top, insets.bottom
        );
        
        let _ = window.eval(&script);
    }
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
mod desktop_impl {
    use super::SafeAreaInsets;
    
    pub fn get_safe_area_insets() -> SafeAreaInsets {
        // 桌面平台不需要安全区域
        SafeAreaInsets { top: 0.0, bottom: 0.0, left: 0.0, right: 0.0 }
    }
    
    pub fn inject_safe_area_to_window(_window: &tauri::WebviewWindow, _insets: SafeAreaInsets) {
        // 桌面平台不需要注入
    }
}

pub fn get_safe_area_insets() -> SafeAreaInsets {
    #[cfg(target_os = "ios")]
    return ios_impl::get_safe_area_insets();
    
    #[cfg(target_os = "android")]
    return android_impl::get_safe_area_insets();
    
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    return desktop_impl::get_safe_area_insets();
}

pub fn inject_safe_area_to_webview(window: &tauri::WebviewWindow) {
    let insets = get_safe_area_insets();
    
    #[cfg(target_os = "ios")]
    ios_impl::inject_safe_area_to_window(window, insets);
    
    #[cfg(target_os = "android")]
    android_impl::inject_safe_area_to_window(window, insets);
    
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    desktop_impl::inject_safe_area_to_window(window, insets);
}
