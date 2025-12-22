#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ButtonAction {
    Step,
    Run,
    RunFast,
    Pause,
    Reset,
    LoadRom,
    LoadSna,
}

pub struct Button {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub action: ButtonAction,
}

impl Button {
    pub fn contains(&self, mx: i32, my: i32) -> bool {
        mx >= self.x && mx < self.x + self.w && my >= self.y && my < self.y + self.h
    }
}

pub fn default_buttons() -> Vec<Button> {
    vec![
        Button { x: 10, y: 10, w: 80, h: 30, action: ButtonAction::Step },
        Button { x: 100, y: 10, w: 80, h: 30, action: ButtonAction::Run },
        Button { x: 190, y: 10, w: 100, h: 30, action: ButtonAction::RunFast },
        Button { x: 310, y: 10, w: 80, h: 30, action: ButtonAction::Pause },
        Button { x: 400, y: 10, w: 80, h: 30, action: ButtonAction::Reset },

        // Fila inferior (carga)
        Button { x: 10, y: 50, w: 120, h: 30, action: ButtonAction::LoadRom },
        Button { x: 140, y: 50, w: 120, h: 30, action: ButtonAction::LoadSna },
    ]
}