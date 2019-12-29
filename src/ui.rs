use crate::component::*;

use amethyst::{
    ecs::{
        Entity, Component, DenseVecStorage, WriteStorage,
    },
    assets::{
        PrefabData,
    },
    ui::{
        EventReceiver, EventRetrigger, EventRetriggerSystemDesc,
        UiEvent, UiEventType, UiWidget, ToNativeWidget,
    },
    error::Error,
};

use serde::{
    Serialize, Deserialize,
};

#[derive(Debug, Clone)]
pub struct CustomUiAction {
    pub target: Entity,
    pub other: Option<Entity>,
    pub event_type: CustomUiActionType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CustomUiActionType {
    KillSelf,
    KillParent,
    DragStartedItem,
    DroppedItem,
    SortInventory,
}

pub type CustomUiActionRetriggerSystemDesc = EventRetriggerSystemDesc<CustomUiActionRetrigger>;

#[derive(Debug)]
pub struct CustomUiActionRetrigger {
    pub on_click_start: Vec<CustomUiAction>,
    pub on_click_stop: Vec<CustomUiAction>,
    pub on_hover_start: Vec<CustomUiAction>,
    pub on_hover_stop: Vec<CustomUiAction>,
    pub on_drop: Vec<CustomUiAction>,
}

impl Component for CustomUiActionRetrigger {
    type Storage = DenseVecStorage<Self>;
}

impl EventRetrigger for CustomUiActionRetrigger {
    type In = UiEvent;
    type Out = CustomUiAction;

    fn apply<R>(&self, event: &Self::In, out: &mut R)
    where
        R: EventReceiver<Self::Out>,
    {
        match event.event_type {
            UiEventType::ClickStart => out.receive(&self.on_click_start),
            UiEventType::ClickStop => out.receive(&self.on_click_stop),
            UiEventType::HoverStart => out.receive(&self.on_hover_start),
            UiEventType::HoverStop => out.receive(&self.on_hover_stop),
            UiEventType::Dropped { dropped_on } => {
                out.receive(
                    &self.on_drop
                    .iter()
                    .map(|a| CustomUiAction { target: a.target, other: dropped_on, event_type: a.event_type })
                    .collect::<Vec<CustomUiAction>>()
                );
            }
            _ => {}
        };
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct CustomUiActionRetriggerData {
    #[serde(default)]
    pub on_click_start: Vec<CustomUiActionType>,
    #[serde(default)]
    pub on_click_stop: Vec<CustomUiActionType>,
    #[serde(default)]
    pub on_hover_start: Vec<CustomUiActionType>,
    #[serde(default)]
    pub on_hover_stop: Vec<CustomUiActionType>,
    #[serde(default)]
    pub on_drop: Vec<CustomUiActionType>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct CustomUiInventoryData {
    pub weight: u8,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct CustomUiItemData {
    pub weight: u8,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct CustomUiPrefabData {
    #[serde(default)]
    pub retriggers: Option<CustomUiActionRetriggerData>,
    #[serde(default)]
    pub inventory: Option<CustomUiInventoryData>,
    #[serde(default)]
    pub item: Option<CustomUiItemData>,
}

impl<'a> PrefabData<'a> for CustomUiPrefabData {
    type SystemData = (
        WriteStorage<'a, CustomUiActionRetrigger>,
        WriteStorage<'a, ComponentItem>,
        WriteStorage<'a, ComponentInventory>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        (retriggers, items, inventories): &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        if let Some(data) = &self.retriggers {
            retriggers.insert(
                entity,
                CustomUiActionRetrigger {
                    on_click_start: data.on_click_start.iter().map(|a| CustomUiAction { target: entity, other: None, event_type: *a }).collect(),
                    on_click_stop: data.on_click_stop.iter().map(|a| CustomUiAction { target: entity, other: None, event_type: *a }).collect(),
                    on_hover_start: data.on_hover_start.iter().map(|a| CustomUiAction { target: entity, other: None, event_type: *a }).collect(),
                    on_hover_stop: data.on_hover_stop.iter().map(|a| CustomUiAction { target: entity, other: None, event_type: *a }).collect(),
                    on_drop: data.on_drop.iter().map(|a| CustomUiAction { target: entity, other: None, event_type: *a}).collect(),
                },
            )?;
        }
        if let Some(data) = &self.inventory {
            inventories.insert(
                entity,
                ComponentInventory {
                    weight: data.weight,
                }
            )?;
        }
        if let Some(data) = &self.item {
            items.insert(
                entity,
                ComponentItem {
                    weight: data.weight,
                    dummy: None,
                }
            )?;
        }

        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum CustomUi {
    CustomItem {
        item: UiWidget<CustomUi>,
        data: CustomUiPrefabData,
    },
}

impl ToNativeWidget for CustomUi {
    type PrefabData = CustomUiPrefabData;

    fn to_native_widget(self, _: Self::PrefabData) -> (UiWidget<CustomUi>, Self::PrefabData) {
        match self {
            CustomUi::CustomItem {
                item,
                data,
            } => {
                (
                    item,
                    data,
                )
            },
        }
    }
}