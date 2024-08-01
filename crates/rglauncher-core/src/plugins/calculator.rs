use crate::plugins::{PluginItemTrait, PluginTrait};
use crate::userinput::UserInput;

use crate::util::scoreutils;
use anyhow::anyhow;
use tracing::info;

pub const TYPE_NAME: &str = "calc";

#[derive(Clone)]
pub struct CalcMsg {}

#[derive(Clone)]
pub struct CalcItem {
    pub formula: String,
    pub result: String,
}

impl PluginItemTrait for CalcItem {
    fn get_score(&self) -> i32 {
        scoreutils::highest(1000)
    }

    fn on_activate(&self) {}

    fn get_type(&self) -> &'static str {
        &TYPE_NAME
    }

    fn get_id(&self) -> &str {
        &TYPE_NAME
    }
}

pub struct CalculatorPlugin {}

impl CalculatorPlugin {
    pub fn new() -> anyhow::Result<Self> {
        info!("Creating Calc Plugin");

        Ok(CalculatorPlugin {})
    }
}

impl PluginTrait for CalculatorPlugin {
    async fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<CalcItem>> {
        if user_input.input.is_empty() {
            return Err(anyhow!("empty input"));
        }

        Ok(vec![meval::eval_str(user_input.input.as_str()).map(
            |res| CalcItem {
                formula: user_input.input.clone(),
                result: res.to_string(),
            },
        )?])
    }

    fn get_type(&self) -> &'static str {
        &TYPE_NAME
    }

    type Item = CalcItem;

    type Msg = CalcMsg;
}
