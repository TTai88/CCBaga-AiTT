// [INPUT]: 外部挂载目录（%APPDATA% 或应用安装目录下的 theme 文件夹）中的 theme.css、logo.png/jpg 及 theme.json。
// [OUTPUT]: 读取并在前端热注入外部皮肤样式与品牌图标，若不存在则回退原生内置资源。
// [POS]: 外部皮肤热加载引擎；无侵入式只读探针，不修改任何核心安全与身份配置。
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ExternalTheme {
    pub css: Option<String>,
    pub logo_base64: Option<String>,
    pub title: Option<String>,
    pub theme_dir: String,
}

fn search_dirs(app: &tauri::AppHandle) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // 1. AppData 漫游目录（安全独立，重新安装软件绝不会被覆盖）：
    // e.g. %APPDATA%\ai.local.requirement-french\theme\
    if let Ok(app_data) = app.path().app_data_dir() {
        dirs.push(app_data.join("theme"));
        dirs.push(app_data);
    }

    // 2. 程序本体安装目录：
    // e.g. D:\CCBaga-A\theme\ 或 D:\CCBaga-A\
    if let Ok(mut exe) = std::env::current_exe() {
        exe.pop();
        dirs.push(exe.join("theme"));
        dirs.push(exe);
    }

    dirs
}

#[tauri::command]
pub fn load_external_theme(app: tauri::AppHandle) -> ExternalTheme {
    let mut result = ExternalTheme::default();
    let dirs = search_dirs(&app);

    for dir in &dirs {
        if !dir.exists() {
            continue;
        }

        // 1. 查找外部 CSS 样式表 (theme.css / custom.css)
        if result.css.is_none() {
            for name in &["theme.css", "custom.css", "style.css"] {
                let p = dir.join(name);
                if p.is_file() {
                    if let Ok(content) = fs::read_to_string(&p) {
                        result.css = Some(content);
                        result.theme_dir = dir.to_string_lossy().to_string();
                        break;
                    }
                }
            }
        }

        // 2. 查找外部 Logo 图片 (logo.png / logo.jpg)
        if result.logo_base64.is_none() {
            for name in &["logo.png", "logo.jpg", "logo.jpeg", "custom-logo.png"] {
                let p = dir.join(name);
                if p.is_file() {
                    if let Ok(bytes) = fs::read(&p) {
                        let mime = if name.ends_with(".png") {
                            "image/png"
                        } else {
                            "image/jpeg"
                        };
                        let b64 = STANDARD.encode(&bytes);
                        result.logo_base64 = Some(format!("data:{mime};base64,{b64}"));
                        break;
                    }
                }
            }
        }

        // 3. 查找外部元信息 (theme.json: {"title": "自定义软件名称"})
        if result.title.is_none() {
            let p = dir.join("theme.json");
            if p.is_file() {
                if let Ok(content) = fs::read_to_string(&p) {
                    #[derive(Deserialize)]
                    struct ThemeMeta {
                        title: Option<String>,
                    }
                    if let Ok(meta) = serde_json::from_str::<ThemeMeta>(&content) {
                        result.title = meta.title;
                    }
                }
            }
        }
    }

    result
}
