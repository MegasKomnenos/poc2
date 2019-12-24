use amethyst::{
    ecs::{
        Entity, Component, DenseVecStorage, WriteStorage,
    },
    assets::{
        PrefabData,
    },
    ui::{
        EventReceiver, EventRetrigger, EventRetriggerSystem, EventRetriggerSystemDesc, UiEvent, UiEventType, UiWidget, ToNativeWidget,
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
    DispInfo,
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

#[derive(Default, Serialize, Deserialize)]
pub struct CustomUiActionRetriggerData {
    pub on_click_start: Vec<CustomUiActionType>,
    pub on_click_stop: Vec<CustomUiActionType>,
    pub on_hover_start: Vec<CustomUiActionType>,
    pub on_hover_stop: Vec<CustomUiActionType>,
}

impl<'a> PrefabData<'a> for CustomUiActionRetriggerData {
    type SystemData = WriteStorage<'a, CustomUiActionRetrigger>;
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        storage.insert(
            entity,
            CustomUiActionRetrigger {
                on_click_start: self.on_click_start.iter().map(|a| CustomUiAction { target: entity, event_type: *a }).collect(),
                on_click_stop: self.on_click_stop.iter().map(|a| CustomUiAction { target: entity, event_type: *a }).collect(),
                on_hover_start: self.on_hover_start.iter().map(|a| CustomUiAction { target: entity, event_type: *a }).collect(),
                on_hover_stop: self.on_hover_stop.iter().map(|a| CustomUiAction { target: entity, event_type: *a }).collect(),
            },
        );

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub enum CustomUi {
    CustomItem {
        item: UiWidget<CustomUi>,
        data: CustomUiActionRetriggerData,
    }
}

impl ToNativeWidget for CustomUi {
    type PrefabData = CustomUiActionRetriggerData;

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
            }
        }
    }
}