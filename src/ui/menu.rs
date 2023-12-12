use crate::{
    save,
    state::{RunState, State},
};

#[derive(PartialEq, Clone, Copy)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Clone, Copy)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

pub fn main_menu(gs: &mut State, ctx: &mut rltk::Rltk) -> MainMenuResult {
    let save_exists = save::has_save_file();
    let run_state = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(
        15,
        rltk::RGB::named(rltk::YELLOW),
        rltk::RGB::named(rltk::BLACK),
        "Daddle-dat Bodan",
    );

    if let RunState::MainMenu {
        menu_selection: selection,
    } = *run_state
    {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(
                24,
                rltk::RGB::named(rltk::MAGENTA),
                rltk::RGB::named(rltk::BLACK),
                "Begin New Game",
            );
        } else {
            ctx.print_color_centered(
                24,
                rltk::RGB::named(rltk::WHITE),
                rltk::RGB::named(rltk::BLACK),
                "Begin New Game",
            );
        }

        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                ctx.print_color_centered(
                    25,
                    rltk::RGB::named(rltk::MAGENTA),
                    rltk::RGB::named(rltk::BLACK),
                    "Load Game",
                );
            } else {
                ctx.print_color_centered(
                    25,
                    rltk::RGB::named(rltk::WHITE),
                    rltk::RGB::named(rltk::BLACK),
                    "Load Game",
                );
            }
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(
                26,
                rltk::RGB::named(rltk::MAGENTA),
                rltk::RGB::named(rltk::BLACK),
                "Quit",
            );
        } else {
            ctx.print_color_centered(
                26,
                rltk::RGB::named(rltk::WHITE),
                rltk::RGB::named(rltk::BLACK),
                "Quit",
            );
        }

        match ctx.key {
            None => {
                return MainMenuResult::NoSelection {
                    selected: selection,
                }
            }
            Some(key) => match key {
                rltk::VirtualKeyCode::Escape => {
                    return MainMenuResult::NoSelection {
                        selected: MainMenuSelection::Quit,
                    }
                }

                rltk::VirtualKeyCode::Up => {
                    let mut next_selection;
                    match selection {
                        MainMenuSelection::NewGame => next_selection = MainMenuSelection::Quit,
                        MainMenuSelection::LoadGame => next_selection = MainMenuSelection::NewGame,
                        MainMenuSelection::Quit => next_selection = MainMenuSelection::LoadGame,
                    }

                    if next_selection == MainMenuSelection::LoadGame && !save_exists {
                        next_selection = MainMenuSelection::NewGame;
                    }

                    return MainMenuResult::NoSelection {
                        selected: next_selection,
                    };
                }

                rltk::VirtualKeyCode::Down => {
                    let mut next_selection;
                    match selection {
                        MainMenuSelection::NewGame => next_selection = MainMenuSelection::LoadGame,
                        MainMenuSelection::LoadGame => next_selection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => next_selection = MainMenuSelection::NewGame,
                    }

                    if next_selection == MainMenuSelection::LoadGame && !save_exists {
                        next_selection = MainMenuSelection::Quit;
                    }

                    return MainMenuResult::NoSelection {
                        selected: next_selection,
                    };
                }

                rltk::VirtualKeyCode::Return => {
                    return MainMenuResult::Selected {
                        selected: selection,
                    }
                }

                _ => {
                    return MainMenuResult::NoSelection {
                        selected: selection,
                    }
                }
            },
        }
    }

    MainMenuResult::NoSelection {
        selected: MainMenuSelection::NewGame,
    }
}
