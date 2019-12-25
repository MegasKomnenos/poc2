use crate::component::*;

use amethyst::{
    ecs::{
        Entity, Component, DenseVecStorage, WriteStorage,
    },
    assets::{
        PrefabData,
    },
    ui::{
        EventReceiver, EventRetrigger, EventRetriggerSystem, EventRetriggerSystemDesc,
        UiImagePrefab, UiTransformData, UiEvent, UiEventType, UiWidget, ToNativeWidget,
    },
    error::Error,
};

use serde::{
    Serialize, Deserialize,
};

#[derive(Debug, Clone)]
pub struct CustomUiAction {
    pub target: Entity,
    pub event_type: CustomUiActionType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CustomUiActionType {
    KillSelf,
    KillParent,
}

pub type CustomUiActionRetriggerSystemDesc = EventRetriggerSystemDesc<CustomUiActionRetrigger>;
pub type CustomUiActionRetriggerSystem = EventRetriggerSystem<CustomUiActionRetrigger>;

#[derive(Debug)]
pub struct CustomUiActionRetrigger {
    pub on_click_start: Vec<CustomUiAction>,
    pub on_click_stop: Vec<CustomUiAction>,
    pub on_hover_start: Vec<CustomUiAction>,
    pub on_hover_stop: Vec<CustomUiAction>,
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
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct CustomUiPrefabData {
    #[serde(default)]
    pub retriggers: Option<CustomUiActionRetriggerData>,
    #[serde(default)]
    pub is_inv_slot: bool,
    #[serde(default)]
    pub is_inv_item: bool,
}

impl<'a> PrefabData<'a> for CustomUiPrefabData {
    type SystemData = (
        WriteStorage<'a, CustomUiActionRetrigger>,
        WriteStorage<'a, ComponentInvSlot>,
        WriteStorage<'a, ComponentInvItem>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        (retriggers, inv_slots, inv_items): &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        if let Some(data) = &self.retriggers {
            retriggers.insert(
                entity,
                CustomUiActionRetrigger {
                    on_click_start: data.on_click_start.iter().map(|a| CustomUiAction { target: entity, event_type: *a }).collect(),
                    on_click_stop: data.on_click_stop.iter().map(|a| CustomUiAction { target: entity, event_type: *a }).collect(),
                    on_hover_start: data.on_hover_start.iter().map(|a| CustomUiAction { target: entity, event_type: *a }).collect(),
                    on_hover_stop: data.on_hover_stop.iter().map(|a| CustomUiAction { target: entity, event_type: *a }).collect(),
                },
            );
        }

        if self.is_inv_slot {
            inv_slots.insert(
                entity,
                ComponentInvSlot,
            );
        }
        if self.is_inv_item {
            inv_items.insert(
                entity,
                ComponentInvItem { name: "Testing".to_string() },
            );
        }

        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum CustomUi {
    Inventory {
        container: UiWidget<CustomUi>,
        slot: UiWidget<CustomUi>,
        slot_data: CustomUiPrefabData,
        slot_num_width: f32,
        slot_num_height: f32,
        slot_change_width: f32,
        slot_change_height: f32,
    },
    CustomItem {
        item: UiWidget<CustomUi>,
        data: CustomUiPrefabData,
    },
}

impl ToNativeWidget for CustomUi {
    type PrefabData = CustomUiPrefabData;

    fn to_native_widget(self, _: Self::PrefabData) -> (UiWidget<CustomUi>, Self::PrefabData) {
        match self {
            CustomUi::Inventory {
                mut container,
                slot,
                slot_data,
                slot_num_width,
                slot_num_height,
                slot_change_width,
                slot_change_height,
            } => {
                if let UiWidget::Container { 
                    children,
                    ..
                } = &mut container {
                    if let UiWidget::Button {
                        transform,
                        button,
                    } = slot {
                        for y in 0..slot_num_height as usize {
                            for x in 0..slot_num_width as usize {
                                let mut transform = transform.clone();
        
                                transform.x += x as f32 * slot_change_width;
                                transform.y += y as f32 * slot_change_height;
        
                                children.push(UiWidget::Custom(Box::new(CustomUi::CustomItem {
                                    item: UiWidget::Button {
                                        transform,
                                        button: button.clone(),
                                    },
                                    data: slot_data.clone(),
                                })));
                            }
                        }
                    }
                }

                (
                    container,
                    Default::default(),
                )
            },
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