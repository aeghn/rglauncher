use lazy_static::lazy_static;

use crate::plugins::{Plugin, PluginResult};
use crate::userinput::UserInput;

use std::option::Option::None;

use crate::util::score_utils;
use std::sync::Mutex;
use tracing::info;

pub const TYPE_ID: &str = "calc";

pub struct CalcMsg {}

pub struct CalcResult {
    pub formula: String,
    pub result: String,
}

impl PluginResult for CalcResult {
    fn score(&self) -> i32 {
        score_utils::highest()
    }

    fn sidebar_icon_name(&self) -> String {
        "calc".to_string()
    }

    fn sidebar_label(&self) -> Option<String> {
        Some("calc".to_string())
    }

    fn sidebar_content(&self) -> Option<String> {
        Some(self.formula.to_string())
    }

    fn on_enter(&self) {}

    fn get_type_id(&self) -> &'static str {
        &TYPE_ID
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
}

pub struct CalculatorPlugin {}

impl CalculatorPlugin {
    pub fn new() -> Self {
        info!("Creating Calc Plugin");

        CalculatorPlugin {}
    }
}

impl Plugin<CalcResult, CalcMsg> for CalculatorPlugin {
    fn handle_msg(&mut self, msg: CalcMsg) {
        todo!()
    }

    fn refresh_content(&mut self) {}

    fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<CalcResult>> {
        Ok(vec![meval::eval_str(user_input.input.as_str()).map(
            |res| CalcResult {
                formula: user_input.input.clone(),
                result: res.to_string(),
            },
        )?])
    }

    fn get_type_id(&self) -> &'static str {
        &TYPE_ID
    }
}