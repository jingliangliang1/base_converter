#![windows_subsystem = "windows"] // 隐藏 Windows 下运行时的 CMD 命令行终端窗口

use eframe::egui;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    // 调整窗口尺寸：加宽到 820px 以确保 32 个框能在一行内完美显示
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 320.0])
            .with_resizable(false)
            .with_title("进制转换器 (32-Bit 单行位图)"),
        ..Default::default()
    };

    eframe::run_native(
        "Base Converter",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(ConverterApp::default()))
        }),
    )
}

/// 配置中文字体，防止中文乱码
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\msyh.ttc") {
        fonts.font_data.insert(
            "msyh".to_owned(),
            Arc::new(egui::FontData::from_owned(font_data)),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "msyh".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "msyh".to_owned());

        ctx.set_fonts(fonts);
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum ActiveBase {
    Bin,
    Oct,
    Dec,
    Hex,
    None,
}

struct ConverterApp {
    value: u32,

    bin_str: String,
    oct_str: String,
    dec_str: String,
    hex_str: String,

    error_msg: String,
}

impl Default for ConverterApp {
    fn default() -> Self {
        let mut app = Self {
            value: 0,
            bin_str: String::from("0"),
            oct_str: String::from("0"),
            dec_str: String::from("0"),
            hex_str: String::from("0"),
            error_msg: String::new(),
        };
        app.update_all_strings();
        app
    }
}

impl ConverterApp {
    /// 根据当前的 u32 value 更新所有文本框
    fn update_all_strings(&mut self) {
        self.bin_str = format!("{:032b}", self.value);
        self.oct_str = format!("{:o}", self.value);
        self.dec_str = format!("{}", self.value);
        self.hex_str = format!("{:X}", self.value);
        self.error_msg.clear();
    }

    /// 解析用户输入的文本框
    fn commit_change(&mut self, base: ActiveBase) {
        let result = match base {
            ActiveBase::Bin => u32::from_str_radix(self.bin_str.trim(), 2),
            ActiveBase::Oct => u32::from_str_radix(self.oct_str.trim(), 8),
            ActiveBase::Dec => self.dec_str.trim().parse::<u32>(),
            ActiveBase::Hex => u32::from_str_radix(
                self.hex_str
                    .trim()
                    .trim_start_matches("0x")
                    .trim_start_matches("0X"),
                16,
            ),
            ActiveBase::None => return,
        };

        match result {
            Ok(parsed_val) => {
                self.value = parsed_val;
                self.update_all_strings();
            }
            Err(_) => {
                self.error_msg = "数值无效或超出 u32 范围 (0 ~ 4294967295)！".to_string();
            }
        }
    }

    /// 切换特定 Bit 位（0 变 1，1 变 0）
    fn toggle_bit(&mut self, bit_index: usize) {
        self.value ^= 1 << bit_index;
        self.update_all_strings();
    }
}

impl eframe::App for ConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("实时进制转换器 (32-Bit)");
            ui.add_space(10.0);

            // ------------------ 32 个 Bit 单行展示 ------------------
            ui.label("二进制 Bit 位图 (Bit 31 -> Bit 0，点击框可切换 0/1):");
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                // 紧凑间距，防止换行
                ui.spacing_mut().item_spacing.x = 2.0;

                for col in 0..32 {
                    let bit_index = 31 - col;
                    let is_set = (self.value & (1 << bit_index)) != 0;
                    let bit_text = if is_set { "1" } else { "0" };

                    // 1 为高亮蓝底白字，0 为灰色暗淡字
                    let button = if is_set {
                        egui::Button::new(
                            egui::RichText::new(bit_text)
                                .monospace()
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(0, 120, 215))
                    } else {
                        egui::Button::new(
                            egui::RichText::new(bit_text)
                                .monospace()
                                .color(egui::Color32::GRAY),
                        )
                    };

                    // 渲染小按钮（宽度 18pt，高度 22pt）
                    if ui.add_sized([18.0, 22.0], button).clicked() {
                        self.toggle_bit(bit_index);
                    }

                    // 每 4 位额外空开 4px 距离，每 8 位（1个字节）再多空一点，方便看位数
                    if col % 4 == 3 && col != 31 {
                        if col % 8 == 7 {
                            ui.add_space(8.0);
                        } else {
                            ui.add_space(4.0);
                        }
                    }
                }
            });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            // ------------------ 各进制文本输入框 ------------------
            let mut commit_base = ActiveBase::None;

            egui::Grid::new("base_grid")
                .num_columns(2)
                .spacing([10.0, 10.0])
                .min_col_width(80.0)
                .show(ui, |ui| {
                    // 1. BIN
                    ui.label("二进制 (BIN):");
                    let res_bin = ui.add(
                        egui::TextEdit::singleline(&mut self.bin_str)
                            .desired_width(600.0)
                            .font(egui::TextStyle::Monospace),
                    );
                    if res_bin.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        commit_base = ActiveBase::Bin;
                    }
                    ui.end_row();

                    // 2. OCT
                    ui.label("八进制 (OCT):");
                    let res_oct = ui.add(
                        egui::TextEdit::singleline(&mut self.oct_str)
                            .desired_width(600.0)
                            .font(egui::TextStyle::Monospace),
                    );
                    if res_oct.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        commit_base = ActiveBase::Oct;
                    }
                    ui.end_row();

                    // 3. DEC
                    ui.label("十进制 (DEC):");
                    let res_dec = ui.add(
                        egui::TextEdit::singleline(&mut self.dec_str)
                            .desired_width(600.0)
                            .font(egui::TextStyle::Monospace),
                    );
                    if res_dec.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        commit_base = ActiveBase::Dec;
                    }
                    ui.end_row();

                    // 4. HEX
                    ui.label("十六进制 (HEX):");
                    let res_hex = ui.add(
                        egui::TextEdit::singleline(&mut self.hex_str)
                            .desired_width(600.0)
                            .font(egui::TextStyle::Monospace),
                    );
                    if res_hex.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        commit_base = ActiveBase::Hex;
                    }
                    ui.end_row();
                });

            if commit_base != ActiveBase::None {
                self.commit_change(commit_base);
            }

            ui.add_space(15.0);

            if !self.error_msg.is_empty() {
                ui.colored_label(egui::Color32::RED, &self.error_msg);
            } else {
                ui.weak("提示：点击上方单行 32-Bit 网格可直接翻转 Bit；在下方文本框输入后按 Enter 完成同步转换。");
            }
        });
    }
}
