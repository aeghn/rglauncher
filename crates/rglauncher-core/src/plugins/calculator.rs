use crate::plugins::{Plugin, PluginResult};
use crate::userinput::UserInput;

use crate::plugins::history::HistoryItem;
use crate::util::score_utils;
use anyhow::anyhow;
use tracing::info;

pub const TYPE_ID: &str = "calc";

#[derive(Clone)]
pub struct CalcMsg {}

pub struct CalcResult {
    pub formula: String,
    pub result: String,
}

impl PluginResult for CalcResult {
    fn score(&self) -> i32 {
        score_utils::highest(1000)
    }

    fn icon_name(&self) -> &str {
        "calc"
    }

    fn name(&self) -> &str {
        self.formula.as_str()
    }

    fn extra(&self) -> Option<&str> {
        None
    }

    fn on_enter(&self) {}

    fn get_type_id(&self) -> &'static str {
        &TYPE_ID
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn get_id(&self) -> &str {
        &TYPE_ID
    }
}

pub struct CalculatorPlugin {}

impl CalculatorPlugin {
    pub fn new() -> anyhow::Result<Self> {
        info!("Creating Calc Plugin");

        Ok(CalculatorPlugin {})
    }
}

impl Plugin<CalcResult, CalcMsg> for CalculatorPlugin {
    fn handle_msg(&mut self, _msg: CalcMsg) {
        todo!()
    }

    fn refresh_content(&mut self) {}

    fn handle_input(
        &self,
        user_input: &UserInput,
        _history: Option<Vec<HistoryItem>>,
    ) -> anyhow::Result<Vec<CalcResult>> {
        if user_input.input.is_empty() {
            return Err(anyhow!("empty input"));
        }

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
