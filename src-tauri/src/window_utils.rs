/// 根据应用数量计算窗口大小
pub fn calculate_window_size(app_count: usize) -> (u32, u32) {
    const ITEM_WIDTH: u32 = 80;
    const ITEM_HEIGHT: u32 = 80;
    const GAP_X: u32 = 12;
    const GAP_Y: u32 = 24;
    const COLUMNS: u32 = 4;
    const INDICATOR_SPACE: u32 = 20; // 小灯空间：bottom: -12px + 小灯6px + 额外缓冲2px
    const PADDING_TOP: u32 = 30;
    const PADDING_BOTTOM: u32 = 30;
    const PADDING_LEFT: u32 = 30; // 增加左右边距防止裁切
    const PADDING_RIGHT: u32 = 30;

    let rows = if app_count == 0 {
        1
    } else {
        ((app_count as f32) / (COLUMNS as f32)).ceil() as u32
    };

    // 宽度固定为4列
    let width = COLUMNS * ITEM_WIDTH + (COLUMNS - 1) * GAP_X + PADDING_LEFT + PADDING_RIGHT;

    // 高度 = 行数×格子高度 + 行间距 + 小灯空间 + 上下边距
    let height = rows * ITEM_HEIGHT
        + rows.saturating_sub(1) * GAP_Y
        + INDICATOR_SPACE // 小灯占用的额外空间
        + PADDING_TOP
        + PADDING_BOTTOM;

    println!(
        "计算窗口尺寸: 应用数={}, 行数={}, 宽度={}, 高度={}",
        app_count, rows, width, height
    );
    (width, height)
}
