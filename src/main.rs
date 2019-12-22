mod misc;
mod state;
mod system;
mod component;
mod asset;
mod ai;
mod map;

use crate::misc::*;
use crate::state::*;
use crate::system::*;

extern crate rand;
extern crate ron;
extern crate voronoi;
extern crate noise;

const NUM_ITEM: usize = 4;
const MAP_SIZE: u32 = 200;

use amethyst::{
    core::transform::TransformBundle,
    prelude::*,
    input::{ InputBundle, StringBindings, },
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{
        RenderUi, UiBundle,
    },
    utils::application_root_dir,
};
use amethyst_tiles::{
    MortonEncoder2D, 
    RenderTiles2D,
};

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let asset_dir = app_root.join("asset");
    let config_dir = app_root.join("config");
    let display_config_path = config_dir.join("display.ron");
    let input_config_path = config_dir.join("input.ron");

    let game_data = GameDataBuilder::default()
        .with(SystemCameraMovement::default(), "Camera Movement System", &[])
        .with(SystemMovement::default(), "Character Movement System", &[])
        .with(SystemMapMode::default(), "Mapmode System", &[])
        .with(SystemTime::default(), "Time System", &[])
        .with(SystemAI::default(), "AI System", &[])
        .with(SystemPrice::default(), "Price System", &[])
        .with_bundle(
            InputBundle::<StringBindings>::new()
                .with_bindings_from_file(input_config_path)?,
        )?
        .with_system_desc(SystemMovementPlayerDesc::default(), "Player Movement System", &["input_system"])
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderTiles2D::<MiscTile, MortonEncoder2D, MiscTileBounds>::default())
                .with_plugin(RenderUi::default())
        )?
        .with_bundle(UiBundle::<StringBindings>::new())?;
        
    let mut game = Application::new(asset_dir, PocLoad::default(), game_data)?;
    game.run();

    Ok(())
}
