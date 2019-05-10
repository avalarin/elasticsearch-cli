#[derive(Default)]
pub struct State {
    pub view: View,
    pub pager: PagerState,
    pub terminal_height: usize
}

pub enum View {
    Pager,
    Help
}

impl Default for View {
    fn default() -> Self {
        View::Pager
    }
}

#[derive(Default)]
pub struct PagerState {
    pub lines: Vec<String>,
    pub top_index: usize,
    pub bottom_index: usize,
    pub has_cropped_items: bool,
    pub scroll_mode: ScrollMode
}

#[derive(PartialEq)]
pub enum ScrollMode {
    ScrollUp,
    ScrollDown
}

impl Default for ScrollMode {
    fn default() -> Self {
        ScrollMode::ScrollUp
    }
}
